use std::{iter::zip, time::Duration};

use rpos::{
    channel::{Channel, Receiver},
    thread_logln,
};

use crate::adc::AdcRawMsg;

pub trait EnumIter
where
    Self: Sized + 'static,
{
    const ITER: &'static [Self];
}

enum JoystickChannel {
    Thrust,
    Direction,
    Aileron,
    Elevator,
}

impl JoystickChannel {
    const ITER: &'static [Self] = &[Self::Thrust, Self::Direction, Self::Aileron, Self::Elevator];
    const STRS: &'static [&'static str] = &["Thrust", "Direction", "Aileron", "Elevator"];
}

#[derive(Default,Clone, Copy)]
struct ChannelInfo {
    index: u8,
    min: u16,
    max: u16,
    rev: bool,
}

enum CalibrateState {
    Idle,
    LowestCheck(u8),
    MinMaxCheck(u8),
}
struct Calibration {
    state: CalibrateState,
    channel_infos: Vec<ChannelInfo>,
    joystick_channel_num: u8,
    rx: Receiver<AdcRawMsg>,
    channel_indexs:Vec<u8>
}

impl Calibration {
    fn new() -> Self {
        Calibration {
            state: CalibrateState::Idle,
            channel_infos: [ChannelInfo::default();JoystickChannel::ITER.len()].to_vec(),
            joystick_channel_num: JoystickChannel::ITER.len() as u8,
            rx: rpos::msg::get_new_rx_of_message::<AdcRawMsg>("adc_raw").unwrap(),
            channel_indexs:Vec::new()
        }
    }

    fn do_sample(&mut self,prewait_seconds:u32,sample_seconds:u32) -> CalSample {
        let mut sample = CalSample::new(&mut self.rx);
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
                thread_logln!("done.");
                self.state = CalibrateState::LowestCheck(0);
            }
            CalibrateState::LowestCheck(x) => {
                thread_logln!(
                    "push channel:{} to lowest side or leftmost side",
                    JoystickChannel::STRS[x as usize]
                );

                let sample = self.do_sample(0,5);
                let chn = sample.find_largest_change_channel();
                self.channel_indexs.push(chn);

                let next_channel = x + 1;
                if next_channel == self.joystick_channel_num {
                    self.state = CalibrateState::MinMaxCheck(0);
                } else {
                    self.state = CalibrateState::LowestCheck(x + 1);
                }
            }
            CalibrateState::MinMaxCheck(_) => todo!(),
        }
    }
}

struct CalSample<'a> {
    list: Vec<AdcRawMsg>,
    rx: &'a mut Receiver<AdcRawMsg>,
}

impl<'a> CalSample<'a> {
    fn new(rx: &'a mut Receiver<AdcRawMsg>) -> Self {
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

fn calibrate_main(argc: u32, argv: *const &str) {}

#[rpos::ctor::ctor]
fn register() {
    rpos::module::Module::register("calibrate", calibrate_main);
}

#[cfg(test)]
mod tests {
    use core::panic;

    use super::*;

    #[test]
    fn test_calsample_average() {
        let mut rx = rpos::msg::get_new_rx_of_message::<AdcRawMsg>("adc_raw").unwrap();
        let mut sample = CalSample::new(&mut rx);
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
        let mut sample = CalSample::new(&mut rx);
        sample.list.push(AdcRawMsg { value: [101,99,103,102] });
        sample.list.push(AdcRawMsg { value: [295,290,301,299] });

        assert_eq!(sample.find_largest_change_channel(),2);
    }

    #[test]
    fn test_calsample_get_min_max(){
        let mut rx = rpos::msg::get_new_rx_of_message::<AdcRawMsg>("adc_raw").unwrap();
        let mut sample = CalSample::new(&mut rx);
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
}
