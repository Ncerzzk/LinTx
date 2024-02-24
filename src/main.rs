use clap::Parser;
use rpos::{
    server_client::{server_init, Client},
};
mod adc;
mod calibrate;
mod mixer;
mod joysticks_test;

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
