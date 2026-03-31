use std::collections::HashMap;

use clap::Parser;
use joydev::{event_codes::AbsoluteAxis, GenericEvent};

use crate::{adc::AdcRawMsg, client_process_args, msgbus::adc_raw_publisher};

#[derive(Parser)]
#[command(name="joy_dev", about = "used for machine with joysticks(/dev/input/js*)", long_about = None)]
struct Cli {
    dev_name: String,
}

fn joy_dev_main(argc: u32, argv: *const &str) {
    let ret = client_process_args::<Cli>(argc, argv);
    if ret.is_none() {
        return;
    }

    let args = ret.unwrap();
    let file = std::fs::File::options()
        .read(true)
        .open(args.dev_name)
        .unwrap();
    let dev = joydev::Device::new(file).unwrap();

    let adc_raw_tx = adc_raw_publisher();
    let chn_map: HashMap<AbsoluteAxis, usize> = [
        (AbsoluteAxis::LeftX, 0),
        (AbsoluteAxis::LeftY, 1),
        (AbsoluteAxis::RightX, 2),
        (AbsoluteAxis::RightY, 3),
    ]
    .into_iter()
    .collect();
    let mut chn_value: [i16; 4] = [0; 4];

    loop {
        let s = dev.get_event().unwrap();
        match s {
            joydev::DeviceEvent::Axis(x) => {
                if let Some(index) = chn_map.get(&x.axis()) {
                    chn_value[*index] = x.value();
                    adc_raw_tx.publish(AdcRawMsg { value: chn_value });
                }
            }
            _ => {}
        }
    }
}

#[rpos::ctor::ctor]
fn register() {
    rpos::module::Module::register("joy_dev", joy_dev_main);
}
