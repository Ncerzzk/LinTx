use std::{fs, io::Read};

use rpos::thread_logln;

use crate::adc::AdcRawMsg;
use crate::calibrate::{
    CalibrationData,
    JoystickChannel::{self, *},
};
use crate::CALIBRATE_FILENAME;

#[derive(Clone)]
pub struct MixerOutMsg{
    pub thrust: u16,
    pub direction: u16,
    pub aileron: u16,
    pub elevator: u16,
}

fn cal_mixout(channel: JoystickChannel, raw: &AdcRawMsg, cal_data: &CalibrationData) -> u16 {
    let channel_cal_info = &cal_data.channel_infos[channel as usize];

    let raw_val = raw.value[channel_cal_info.index as usize]
        .clamp(channel_cal_info.min, channel_cal_info.max) as i32;

    let mut ret = (raw_val - channel_cal_info.min as i32) as u32 * 10000
        / (channel_cal_info.max as i32 - channel_cal_info.min as i32) as u32;

    if channel_cal_info.rev {
        ret = 10000 - ret;
    }

    ret as u16
}

fn mixer_main(_argc: u32, _argv: *const &str) {
    let rx = rpos::msg::get_new_rx_of_message::<AdcRawMsg>("adc_raw").unwrap();
    let tx = rpos::msg::get_new_tx_of_message::<MixerOutMsg>("mixer_out").unwrap();
    let mut toml_str = String::new();
    if let Ok(mut file) = fs::File::open(CALIBRATE_FILENAME) {
        file.read_to_string(&mut toml_str).unwrap();
    } else {
        thread_logln!("no joystick.toml found. please calibrate joysticks first!");
        return;
    }

    let cal_data = toml::from_str::<CalibrationData>(toml_str.as_str()).unwrap();

    rx.register_callback("mixer_callback", move |x| {
        let mixer_out = MixerOutMsg {
            thrust: cal_mixout(Thrust, x, &cal_data),
            direction: cal_mixout(Direction, x, &cal_data),
            aileron: cal_mixout(Aileron, x, &cal_data),
            elevator: cal_mixout(Elevator, x, &cal_data),
        };
        tx.send(mixer_out);
    });
}

#[rpos::ctor::ctor]
fn register() {
    rpos::msg::add_message::<MixerOutMsg>("mixer_out");
    rpos::module::Module::register("mixer", mixer_main);
}

#[cfg(test)]
mod tests {
    use crate::calibrate::ChannelInfo;

    use super::*;
    use ads1x1x::ChannelId;
    use rand::prelude::*;

    #[test]
    fn test_cal_mixout() {
        let mut rng = thread_rng();
        let mut get_random_channel_value = || rng.gen_range(300..1400) as i16;
        let mut adc_raw = AdcRawMsg {
            value: [
                500,
                100,
                1600,
                get_random_channel_value(),
            ],
        };
        let mut cal_data = CalibrationData {
            channel_infos: [
                ChannelInfo {
                    name: "thrust".to_string(),
                    index: 0,
                    min: 200,
                    max: 1500,
                    rev: false,
                },
                ChannelInfo {
                    name: "direction".to_string(),
                    index: 1,
                    min: 200,
                    max: 1500,
                    rev: false,
                },
                ChannelInfo {
                    name: "aliron".to_string(),
                    index: 2,
                    min: 200,
                    max: 1500,
                    rev: false,
                },
                ChannelInfo {
                    name: "ele".to_string(),
                    index: 3,
                    min: 200,
                    max: 1500,
                    rev: false,
                },
            ]
            .to_vec(),
            channel_indexs: [0; 4].to_vec(),
        };

        assert_eq!(cal_mixout(JoystickChannel::Thrust, &adc_raw, &cal_data), ((500 - 200) as u32 *10000  / (1500 - 200) )as u16);
        assert_eq!(cal_mixout(JoystickChannel::Direction, &adc_raw, &cal_data), ((200 - 200) as u32 *10000  / (1500 - 200) )as u16);
        assert_eq!(cal_mixout(JoystickChannel::Aileron, &adc_raw, &cal_data), ((1500 - 200) as u32 *10000  / (1500 - 200) )as u16);

        for _ in 0..1000{
            assert!(cal_mixout(JoystickChannel::Elevator, &adc_raw, &cal_data) <= 10000 );
            adc_raw.value[3] = get_random_channel_value();
        }


        cal_data.channel_infos[0].rev = true;
        assert_eq!(cal_mixout(JoystickChannel::Thrust, &adc_raw, &cal_data), 10000 - ((500 - 200) as u32 *10000  / (1500 - 200) )as u16);


    }
}
