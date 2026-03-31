use std::{fs::File, io::Write};

use crate::msgbus::mixer_out_subscriber;

struct GamePad{
    fd:File
}

impl GamePad{
    fn new(device_name:&'static str) -> GamePad{
        let file = std::fs::OpenOptions::new().read(false).write(true).create_new(false).open(device_name).unwrap();

        let gamepad = GamePad{
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
    let mut rx = mixer_out_subscriber();
    loop{
        let mix_out = rx.read();
        game_pad.update_report(0, &[mix_out.thrust,mix_out.direction,mix_out.aileron,mix_out.elevator]);
    }
    
}

#[rpos::ctor::ctor]
fn register() {
    rpos::module::Module::register("gamepad", gamepad_main);
}
