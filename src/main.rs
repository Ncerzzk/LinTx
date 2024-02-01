use std::{sync::{Arc, RwLock}, fs::File, io::Write};

use linux_embedded_hal::I2cdev;
use nb::block;

use ads1x1x::{channel, Ads1x1x, SlaveAddr};

struct GamePad{
    report:GamePadReport,
    fd:File
}

#[derive(Default)]
struct GamePadReport{
    button_status:u8,
    channels_status:[i8;4]
}

impl GamePad{
    fn new(device_name:&'static str) -> GamePad{
        let file = std::fs::OpenOptions::new().read(false).write(true).create_new(false).open(device_name).unwrap();

        let gamepad = GamePad{
            report:GamePadReport::default(),
            fd:file
        };

        gamepad
    }

    // channel value:-1 ~ 1
    fn update_report(&mut self,button_status:u8,channel:&[f32;4]){
        let mut buf: [u8;5]=[0;5];
        for (index,value) in channel.iter().enumerate(){
            buf[index+1] = (value * (i8::MAX as f32)) as i8 as u8;
        } 
        buf[0] = button_status;
        self.fd.write(&buf).unwrap();
        self.fd.flush().unwrap();

    }
}

fn main() {
    const ADC_MIN:u16=100;
    const ADC_MAX:u16=1500;
    const ADC_MIDDLE:u16 = (ADC_MAX + ADC_MIN)/2;
    let dev = I2cdev::new("/dev/i2c-0").unwrap();
    let address = SlaveAddr::default();
    let mut adc = Ads1x1x::new_ads1015(dev, address);
    adc.set_full_scale_range(ads1x1x::FullScaleRange::Within4_096V).unwrap();
    adc.set_data_rate(ads1x1x::DataRate12Bit::Sps3300).unwrap();

    let mut game_pad = GamePad::new("/dev/hidg0");

    loop{
        let value = [
            block!(adc.read(channel::SingleA0)).unwrap(),
            block!(adc.read(channel::SingleA1)).unwrap(),
            block!(adc.read(channel::SingleA2)).unwrap(),
            block!(adc.read(channel::SingleA3)).unwrap()
        ];


        let value = value.map(|x|{
            let percent = ((x as u16 - ADC_MIN) as f32) / ((ADC_MAX - ADC_MIN) as f32);
            percent*2.0 - 1.0
        });

        game_pad.update_report(0, &value);


    }
    // get I2C device back
    let _dev = adc.destroy_ads1015();
}