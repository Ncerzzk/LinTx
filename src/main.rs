use std::sync::{Arc, RwLock};

use linux_embedded_hal::I2cdev;
use nb::block;

use ads1x1x::{channel, Ads1x1x, SlaveAddr};

fn main() {
    let dev = I2cdev::new("/dev/i2c-0").unwrap();
    let address = SlaveAddr::default();
    let mut adc = Ads1x1x::new_ads1015(dev, address);
    adc.set_full_scale_range(ads1x1x::FullScaleRange::Within4_096V).unwrap();
    adc.set_data_rate(ads1x1x::DataRate12Bit::Sps3300).unwrap();

    let cnt =Arc::new(RwLock::new(0));
    let cnt2 = cnt.clone();

    if let Ok(mut x) = adc.into_continuous(){
        x.read();
    }

    std::thread::spawn(move ||{
        loop{
            println!("cnt:{}",cnt2.read().unwrap());
            *(cnt2.write().unwrap())=0;
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    loop{
        let value = [
            block!(adc.read(channel::SingleA0)).unwrap(),
            block!(adc.read(channel::SingleA1)).unwrap(),
            block!(adc.read(channel::SingleA2)).unwrap(),
            block!(adc.read(channel::SingleA3)).unwrap()
        ];
        let mut cnt_unlock = cnt.write().unwrap();
        *cnt_unlock = *cnt_unlock + 1;

        //println!("Measurement: {:?}", value);
    }
    // get I2C device back
    let _dev = adc.destroy_ads1015();
}