use std::iter::zip;

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
}

impl Calibration {
    fn new() -> Self {
        Calibration {
            state: CalibrateState::Idle,
            channel_infos: Vec::new(),
            joystick_channel_num: 4,
            rx: rpos::msg::get_new_rx_of_message::<AdcRawMsg>("adc_raw").unwrap(),
        }
    }

    fn do_sample(&mut self) -> CalSample {
        let mut sample = CalSample::new(&mut self.rx);
        std::thread::sleep(std::time::Duration::from_secs(3));
        sample.sample_in_seconds(2);
        sample
    }

    fn do_step(&mut self) {
        match self.state {
            CalibrateState::Idle => {
                thread_logln!("start calibrate joysticks!");
                thread_logln!("step 0:push joysticks to center.");
                let sample = self.do_sample();
                self.state = CalibrateState::LowestCheck(0);
            }
            CalibrateState::LowestCheck(x) => {
                thread_logln!(
                    "push channel:{} to lowest side or leftmost side",
                    JoystickChannel::STRS[x as usize]
                );
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
    cnt: u32,
    rx: &'a mut Receiver<AdcRawMsg>,
}

impl<'a> CalSample<'a> {
    fn new(rx: &'a mut Receiver<AdcRawMsg>) -> Self {
        CalSample {
            list: Vec::new(),
            cnt: 0,
            rx,
        }
    }

    fn get_average(&self) -> AdcRawMsg {
        let mut ret = self
            .list
            .iter()
            .fold(AdcRawMsg { value: [0; 4] }, |acc, x| {
                let mut tmp = AdcRawMsg { value: [0; 4] };
                for (index, (a, b)) in acc.value.iter().zip(x.value).enumerate() {
                    tmp.value[index] = a + b;
                }
                tmp
            });
        for i in ret.value.iter_mut() {
            *i = (*i) / self.cnt as i16;
        }
        ret
    }

    fn sample_in_seconds(&mut self, seconds: u32) {
        let start_time = std::time::Instant::now();
        loop {
            let data = self.rx.read();
            self.list.push(data);
            self.cnt += 1;

            if start_time.elapsed().as_secs() >= seconds as u64 {
                break;
            }
        }
    }

    fn sample_by_counts(&mut self, count: u32) {
        while self.cnt < count {
            let data = self.rx.read();
            self.list.push(data);
            self.cnt += 1;
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
    fn test_calsample() {
        let mut rx = rpos::msg::get_new_rx_of_message::<AdcRawMsg>("adc_raw").unwrap();
        let mut sample = CalSample::new(&mut rx);
        sample.list.push(AdcRawMsg { value: [100; 4] });
        sample.list.push(AdcRawMsg { value: [200; 4] });
        sample.cnt = 2;

        let average = sample.get_average();

        for i in average.value {
            assert_eq!(i, 150);
        }
    }
}
