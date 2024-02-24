use std::{time::Duration, io::Write};

use rpos::{
    channel::Receiver,
    thread_logln
};

use crate::{adc::AdcRawMsg, CALIBRATE_FILENAME};

pub trait EnumIter
where
    Self: Sized + 'static,
{
    const ITER: &'static [Self];
}

pub enum JoystickChannel {
    Thrust,
    Direction,
    Aileron,
    Elevator,
}

impl JoystickChannel {
    pub const ITER: &'static [Self] = &[Self::Thrust, Self::Direction, Self::Aileron, Self::Elevator];
    pub const STRS: &'static [&'static str] = &["Thrust", "Direction", "Aileron", "Elevator"];
}

#[derive(Default,Clone,Debug,serde::Serialize,serde::Deserialize)]
pub struct ChannelInfo {
    pub name: String,
    pub index: u8,
    pub min: i16,
    pub max: i16,
    pub rev: bool,
}

#[derive(serde::Serialize,serde::Deserialize)]
pub struct CalibrationData{
    pub channel_infos:Vec<ChannelInfo>,
    pub channel_indexs:Vec<u8>
}

enum CalibrateState {
    Idle,
    LowestCheck(u8),
    MinMaxCheck,
    Finish
}
struct Calibration {
    state: CalibrateState,
    data:CalibrationData,
    rx: Receiver<AdcRawMsg>,
}

impl Calibration {
    fn new() -> Self {
        Calibration {
            state: CalibrateState::Idle,
            rx: rpos::msg::get_new_rx_of_message::<AdcRawMsg>("adc_raw").unwrap(),
            data: CalibrationData{
                channel_infos: Vec::new(),
                channel_indexs:Vec::new()
            }
        }
    }

    fn do_sample(&mut self,prewait_seconds:u32,sample_seconds:u32) -> CalSample {
        let mut sample = CalSample::new(self.rx.clone());
        std::thread::sleep(Duration::from_secs(prewait_seconds as u64));
        sample.sample_in_seconds(sample_seconds);
        sample
    }

    fn do_step(&mut self) {
        match self.state {
            CalibrateState::Idle => {
                thread_logln!("start calibrate joysticks!");
                thread_logln!("step 0:push joysticks to center.");
                std::thread::sleep(std::time::Duration::from_secs(5));
                self.state = CalibrateState::LowestCheck(0);
            }
            CalibrateState::LowestCheck(x) => {
                thread_logln!(
                    "step {}:push channel:{} to lowest side or leftmost side", x+1,
                    JoystickChannel::STRS[x as usize]
                );

                let sample = self.do_sample(2,3);
                let chn = sample.find_largest_change_channel();
                self.data.channel_indexs.push(chn);

                let next_channel = x + 1;
                if next_channel == JoystickChannel::ITER.len() as u8 {
                    self.state = CalibrateState::MinMaxCheck;
                } else {
                    self.state = CalibrateState::LowestCheck(next_channel);
                }
            }
            CalibrateState::MinMaxCheck =>{
                thread_logln!("last step: rotate all your joysticks to check min and max value.");
                let sample = self.do_sample(0, 10);
                for (index,_) in JoystickChannel::ITER.iter().enumerate(){
                    let channel_index = self.data.channel_indexs[index];
                    let min = sample.get_min_of_channel(channel_index);
                    let max = sample.get_max_of_channel(channel_index);
                    let chn = ChannelInfo{
                        index: channel_index,
                        min,
                        max,
                        rev:false,
                        name:JoystickChannel::STRS[index].to_string(),
                    };
                    self.data.channel_infos.push(chn);
                }
                self.state = CalibrateState::Finish;
            },
            CalibrateState::Finish =>{
                thread_logln!("finished.");
                for (index,channel_name) in JoystickChannel::STRS.iter().enumerate(){
                    thread_logln!("{}:{:?}",channel_name,self.data.channel_infos[index]);
                }
            }
        }
    }
}

struct CalSample{
    list: Vec<AdcRawMsg>,
    rx: Receiver<AdcRawMsg>,
}

impl CalSample {
    fn new(rx:Receiver<AdcRawMsg>) -> Self {
        CalSample {
            list: Vec::new(),
            rx,
        }
    }

    fn get_min_of_channel(&self,channel_index:u8)->i16{
        let a = self.list.iter().min_by_key(|x|{
            x.value[channel_index as usize]
        }).unwrap();
        a.value[channel_index as usize]
    }

    fn get_max_of_channel(&self,channel_index:u8)->i16{
        let a = self.list.iter().max_by_key(|x|{
            x.value[channel_index as usize]
        }).unwrap();
        a.value[channel_index as usize]      
    }

    fn get_average(&self) -> AdcRawMsg {
        let mut ret = self
            .list
            .iter()
            .fold(AdcRawMsg::default(), |acc, x| {
                let mut tmp = AdcRawMsg::default();
                for (index, (a, b)) in acc.value.iter().zip(x.value).enumerate() {
                    tmp.value[index] = a + b;
                }
                tmp
            });
        for i in ret.value.iter_mut() {
            *i = (*i) / self.list.len() as i16;
        }
        ret
    }

    fn find_largest_change_channel(&self)->u8{
        let cmp_func = |a: &&AdcRawMsg,b: &&AdcRawMsg|{
            let sum_a:i16 = a.value.iter().sum();
            let sum_b:i16 = b.value.iter().sum();
            sum_a.cmp(&sum_b) 
        };

        let max = self.list.iter().max_by(cmp_func).unwrap();
        let min = self.list.iter().min_by(cmp_func).unwrap();

        let mut max_diff=0;
        let mut ret:u8=0;
        for (index,(a,b)) in max.value.iter().zip(min.value).enumerate(){
            let sub = (a-b).abs();
            if sub > max_diff{
                max_diff = sub;
                ret = index as u8;
            }
        }
        ret
    }

    fn sample_in_seconds(&mut self, seconds: u32) {
        let start_time = std::time::Instant::now();
        loop {
            let data = self.rx.read();
            self.list.push(data);

            if start_time.elapsed().as_secs() >= seconds as u64 {
                break;
            }
        }
    }

    fn sample_by_counts(&mut self, count: u32) {
        while (self.list.len() as u32) < count {
            let data = self.rx.read();
            self.list.push(data);
        }
    }
}

fn calibrate_main(_argc: u32, _argv: *const &str) {
    let mut cal = Calibration::new();
    loop{
        match cal.state{
            CalibrateState::Finish => {
                cal.do_step();
                break
            },
            _ => cal.do_step()
        }
    }
    let mut file = std::fs::OpenOptions::new().read(false).write(true).create_new(true).open(CALIBRATE_FILENAME).unwrap();
    let str_write = toml::to_string(&cal.data).unwrap();
    file.write(str_write.as_bytes()).unwrap();
}

#[rpos::ctor::ctor]
fn register() {
    rpos::module::Module::register("calibrate", calibrate_main);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    #[derive(Serialize)]
    struct SaveInfo{
        a:Vec<u8>,
        b:Vec<u8>
    }

    #[test]
    fn test_save_cal_data(){
        let mut a = CalibrationData{
            channel_indexs:[0,1,2,3].to_vec(),
            channel_infos:Vec::new()
        };
        a.channel_infos.push(ChannelInfo::default());
        a.channel_infos.push(ChannelInfo::default());

        let b = toml::to_string(&a).unwrap();
        thread_logln!("{}",b);
    }
    #[test]
    fn test_calsample_average() {
        let mut rx = rpos::msg::get_new_rx_of_message::<AdcRawMsg>("adc_raw").unwrap();
        let mut sample = CalSample::new(rx.clone());
        sample.list.push(AdcRawMsg { value: [100; 4] });
        sample.list.push(AdcRawMsg { value: [200; 4] });

        let average = sample.get_average();

        for i in average.value {
            assert_eq!(i, 150);
        }
    }

    #[test]
    fn test_calsample_find_largest_changes_channel(){
        let mut rx = rpos::msg::get_new_rx_of_message::<AdcRawMsg>("adc_raw").unwrap();
        let mut sample = CalSample::new(rx.clone());
        sample.list.push(AdcRawMsg { value: [101,99,103,102] });
        sample.list.push(AdcRawMsg { value: [295,290,301,299] });

        assert_eq!(sample.find_largest_change_channel(),2);
    }

    #[test]
    fn test_calsample_get_min_max(){
        let mut rx = rpos::msg::get_new_rx_of_message::<AdcRawMsg>("adc_raw").unwrap();
    let mut sample = CalSample::new(rx.clone());
        const CHANNEL_NUM: usize = JoystickChannel::ITER.len();
        sample.list.push(AdcRawMsg { value: [50;CHANNEL_NUM] });
        sample.list.push(AdcRawMsg { value: [100;CHANNEL_NUM] });
        sample.list.push(AdcRawMsg { value: [200;CHANNEL_NUM] });
        sample.list.push(AdcRawMsg { value: [300;CHANNEL_NUM] });
        sample.list.push(AdcRawMsg { value: [400;CHANNEL_NUM] });
        sample.list.push(AdcRawMsg { value: [500;CHANNEL_NUM] });

        for i in 0..CHANNEL_NUM{
            assert_eq!(sample.get_min_of_channel(i as u8),50);
            assert_eq!(sample.get_max_of_channel(i as u8),500);
        }
        
    }

    #[test]
    fn test_calibrate(){
        let mut cal = Calibration::new();
        std::thread::spawn(||{
            let tx = rpos::msg::get_new_tx_of_message::<AdcRawMsg>("adc_raw").unwrap();
            
            loop{
                let msg = AdcRawMsg::default();
                tx.send(msg);
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });

        loop{
            match cal.state{
                CalibrateState::Finish => {
                    cal.do_step();
                    break
                },
                _ => cal.do_step()
            }
        }
    }
}
