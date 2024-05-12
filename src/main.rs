use clap::Parser;
use rpos::{
    server_client::{server_init, Client},
};
mod adc;
mod calibrate;
mod mixer;
mod elrs_tx;
mod joy_dev;
mod joysticks_test;
mod gampad;


pub const CALIBRATE_FILENAME: &str = "joystick.toml";

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help(true))]
struct Cli {
    #[arg(long)]
    server: bool,

    /// commands send by clients.
    #[arg(value_name = "client commands")]
    other: Option<Vec<String>>,
}

pub fn client_process_args<T:clap::Parser>(
    argc: u32,
    argv: *const &str
) -> Option<T> {

    let argv = unsafe { std::slice::from_raw_parts(argv, argc as usize) };

    let ret = T::try_parse_from(argv);

    if ret.is_err() {
        let help_str = T::command().render_help();
        rpos::thread_logln!("{}", help_str);
        return None
    }
    ret.ok()
}

fn main() {
    const SOCKET_PATH: &str = "./rpsocket";
    let cli = Cli::parse();

    if cli.server {
        let hello_txt = r"
        __    _     ______    
       / /   (_)___/_  __/  __
      / /   / / __ \/ / | |/_/
     / /___/ / / / / / _>  <  
    /_____/_/_/ /_/_/ /_/|_|  ";

        println!("{hello_txt}");

        println!("Built from branch={} commit={} dirty={} source_timestamp={}",
            env!("GIT_BRANCH"),
            env!("GIT_COMMIT"),
            env!("GIT_DIRTY"),
            env!("SOURCE_TIMESTAMP"),
        );

        server_init(SOCKET_PATH).unwrap();
    } else {
        let mut client = Client::new(SOCKET_PATH).unwrap();
        client.send_str(cli.other.unwrap().join(" ").as_str());
        client.block_read();
    }
}
