use std::{fs, io::Read};

use crate::adc::AdcRawMsg;
use crate::calibrate::{CalibrationData,JoystickChannel::{self,*}};

#[derive(Clone)]
struct MixerOutMsg{
    thrust:u16,
    direction:u16,
    aileron:u16,
    elevator:u16
}

fn cal_mixout(channel:JoystickChannel,raw: &AdcRawMsg, cal_data:&CalibrationData) -> u16{
    let channel_cal_info = cal_data.channel_infos[channel as usize];

    let raw_val = raw.value[channel_cal_info.index as usize].clamp(channel_cal_info.min, channel_cal_info.max);

    let mut ret= (raw_val - channel_cal_info.min) as u32 * 10000 / (channel_cal_info.max - channel_cal_info.min) as u32;

    if channel_cal_info.rev{
        ret = 10000 - ret;
    }

    ret as u16
}

fn mixer_main(argc: u32, argv: *const &str) {
    let rx = rpos::msg::get_new_rx_of_message::<AdcRawMsg>("adc_raw").unwrap();
    let tx = rpos::msg::get_new_tx_of_message::<MixerOutMsg>("mixer_out").unwrap();
    let mut toml_str = String::new();
    fs::File::open("joystick.toml").unwrap().read_to_string(&mut toml_str).unwrap();
    let cal_data = toml::from_str::<CalibrationData>(toml_str.as_str()).unwrap();

    rx.register_callback("mixer_callback", move |x|{
        let mixer_out = MixerOutMsg{
            thrust:cal_mixout(Thrust, x, &cal_data),
            direction:cal_mixout(Direction, x, &cal_data),
            aileron:cal_mixout(Aileron, x, &cal_data),
            elevator:cal_mixout(Elevator, x, &cal_data),
        };
        tx.send(mixer_out);
    })
}

#[rpos::ctor::ctor]
fn register() {
    rpos::msg::add_message::<MixerOutMsg>("mixer_out");
    rpos::module::Module::register("mixer", mixer_main);
}