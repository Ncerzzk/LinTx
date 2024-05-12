use std::time::Duration;

use clap::Parser;
use crc::{Crc, CRC_8_DVB_S2};
use crsf::{PacketAddress, PacketType, RawPacket, RcChannels};
use rpos::{msg::get_new_rx_of_message, pthread_scheduler::SchedulePthread, thread_logln};

use crate::{client_process_args, mixer::MixerOutMsg};

#[derive(Parser)]
#[command(name="erls_tx", about = None, long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = 115200)]
    baudrate: u32,

    dev_name: String,
}


fn new_rc_channel_packet(channel_vals: &[u16; 16]) -> RawPacket {
    let chn = crsf::RcChannels(*channel_vals);
    let packet = crsf::Packet::RcChannels(chn);
    packet.into_raw(PacketAddress::Transmitter)
}

fn gen_magic_packet() -> [u8; 8] {
    let mut data = [0; 8];
    let crc8_alg = Crc::<u8>::new(&CRC_8_DVB_S2);
    data[0] = 0xEE; //ELRS_ADDRESS
    data[1] = 6;
    data[2] = 0x2D; //TYPE_SETTINGS_WRITE
    data[3] = 0xEE;
    data[4] = 0xEA; //  Radio Transmitter
    data[5] = 0x1;
    data[6] = 0x00;
    data[7] = crc8_alg.checksum(&data[2..7]);
    data
}

#[inline]
fn mxier_out_2_crsf(val:u16) -> u16{
    (val as u32  * (crsf::RcChannels::CHANNEL_VALUE_MAX - crsf::RcChannels::CHANNEL_VALUE_MIN) as u32 / 10000 + crsf::RcChannels::CHANNEL_VALUE_MIN as u32) as u16
}

fn elrs_tx_main(argc: u32, argv: *const &str) {
    let arg_ret = client_process_args::<Cli>(argc, argv);
    if arg_ret.is_none() {
        return;
    }

    let args = arg_ret.unwrap();

    let dev_name = &args.dev_name;
    let serial = serialport::new(dev_name, args.baudrate);
    let mut dev = serial.timeout(Duration::from_millis(1000)).open().unwrap();
    let mut rx = get_new_rx_of_message::<MixerOutMsg>("mixer_out").unwrap();

    let magic_cmd = gen_magic_packet();
    for _ in 0..10 {
        dev.write(&magic_cmd).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    thread_logln!("elrs_tx start!");

    SchedulePthread::new_simple(Box::new(move |_| {
        let mut crsf_chn_values:[u16;16] = [0;16];
        loop {
            let msg = rx.read();
            crsf_chn_values[0] = mxier_out_2_crsf(msg.aileron);
            crsf_chn_values[1] = mxier_out_2_crsf(msg.elevator);
            crsf_chn_values[2] = mxier_out_2_crsf(msg.thrust);
            crsf_chn_values[3] = mxier_out_2_crsf(msg.direction);
            let raw_packet = new_rc_channel_packet(&crsf_chn_values);
            dev.write(raw_packet.data()).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }));
}

#[rpos::ctor::ctor]
fn register() {
    rpos::module::Module::register("elrs_tx", elrs_tx_main);
}
