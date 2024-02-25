use std::{fs::File, io::Write};

use rpos::msg::get_new_rx_of_message;

use crate::mixer::MixerOutMsg;

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

    // channel value:0 ~ 10000
    fn update_report(&mut self,button_status:u8,channel:&[u16;4]){
        let mut buf: [u8;5]=[0;5];
        for (index,value) in channel.iter().enumerate(){
            let temp = ((*value as i32 - 5000) * i8::MAX as i32) / 5000 as i32;
            buf[index+1] = temp as i8 as u8;
        } 
        buf[0] = button_status;
        self.fd.write(&buf).unwrap();
        self.fd.flush().unwrap();

    }
}

fn gamepad_main(_argc: u32, _argv: *const &str) {
    let mut game_pad = GamePad::new("/dev/hidg0");
    let mut rx = get_new_rx_of_message::<MixerOutMsg>("mixer_out").unwrap();
    loop{
        let mix_out = rx.read();
        game_pad.update_report(0, &[mix_out.thrust,mix_out.direction,mix_out.aileron,mix_out.elevator]);
    }
    
}

#[rpos::ctor::ctor]
fn register() {
    rpos::module::Module::register("gamepad", gamepad_main);
}
