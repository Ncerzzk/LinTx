use rpos::{thread_log, msg::get_new_rx_of_message};

use crate::mixer::MixerOutMsg;

fn channel_out(mixout:&MixerOutMsg){
    thread_log!("\x1b[2KThrust:{}\n",mixout.thrust);
    thread_log!("\x1b[2Kdirection:{}\n",mixout.direction);
    thread_log!("\x1b[2Kaileron:{}\n",mixout.aileron);
    thread_log!("\x1b[2Kelevator:{}\n",mixout.elevator);
    thread_log!("\x1b[4A");
    
}
fn joysticks_test_main(_argc: u32, _argv: *const &str) {
    let mut rx = get_new_rx_of_message::<MixerOutMsg>("mixer_out").unwrap();
    loop{
        channel_out(&rx.read());
    }
}

#[rpos::ctor::ctor]
fn register() {
    rpos::module::Module::register("joysticks_test", joysticks_test_main);
}

#[cfg(test)]
mod tests{

    use super::*;
    #[test]
    fn test_channel_out(){
        for i in 0..100 as u16{
            let mixout = MixerOutMsg{
                thrust:i*100,
                direction:i*100,
                elevator:i*100,
                aileron:i*100,
            };
            channel_out(&mixout);
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

    }
}