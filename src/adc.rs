
use rpos::thread_logln;
use std::{io::Write};

use linux_embedded_hal::I2cdev;
use nb::block;

use ads1x1x::{channel, Ads1x1x, SlaveAddr};

#[derive(Debug,Clone,Copy,Default)]
pub struct AdcRawMsg{
    pub value:[i16;4]
}

fn adc_main(_argc: u32, _argv: *const &str) {
    let dev = I2cdev::new("/dev/i2c-0").unwrap();
    let address = SlaveAddr::default();
    let mut adc = Ads1x1x::new_ads1015(dev, address);
    adc.set_full_scale_range(ads1x1x::FullScaleRange::Within4_096V).unwrap();
    adc.set_data_rate(ads1x1x::DataRate12Bit::Sps3300).unwrap();

    let adc_raw_tx = rpos::msg::get_new_tx_of_message::<AdcRawMsg>("adc_raw").unwrap();

    thread_logln!("adc thread started!");

    loop{
        let value = [
            block!(adc.read(channel::SingleA0)).unwrap(),
            block!(adc.read(channel::SingleA1)).unwrap(),
            block!(adc.read(channel::SingleA2)).unwrap(),
            block!(adc.read(channel::SingleA3)).unwrap()
        ];

        adc_raw_tx.send(AdcRawMsg{ value });
    }
    // get I2C device back
    #[warn(unreachable_code)]
    let _dev = adc.destroy_ads1015();
}

#[rpos::ctor::ctor]
fn register() {
    rpos::msg::add_message::<AdcRawMsg>("adc_raw");
    rpos::module::Module::register("adc", adc_main);
}