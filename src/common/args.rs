

use clap::Parser;
use clap::ArgAction;

use crate::Protocol;

#[derive(Parser, Debug)]
#[command(author, version, about = "Quick-Serve", long_about = "Instant file serving made easy")]
pub struct Cli {

    // TODO: Still have to figure a way to **not** show the 
    // headless option when running headless
    #[arg(
        help = "Headless",
        long, required = false,
        action = ArgAction::SetTrue,
    )] pub headless: bool,

    #[arg(
        help = "Bind IP",
        short, long, required = false,
        default_value = "127.0.0.1",
        value_name = "IP",
        require_equals = true,
    )] pub bind_ip: String,

    #[arg(
        help = "Directory to serve",
        short = 'd', long, required = false,
        default_value = "/tmp/",
        value_name = "PATH",
        require_equals = true,
    )] pub serve_dir: String,

    #[arg(
        help = "Verbose logging",
        short, long, required = false,
        action = clap::ArgAction::Count,
    )] pub verbose: u8,

    #[arg(
        default_missing_value = Protocol::Http.get_default_port().to_string(),
        help = format!("Start the HTTP server [default port: {}]", Protocol::Http.get_default_port().to_string()),
        long, required = false, 
        num_args = 0..=1,
        require_equals = true,
        value_name = "PORT",
    )] pub http: Option<u32>,

    #[arg(
        default_missing_value = Protocol::Ftp.get_default_port().to_string(),
        help = format!("Start the FTP server [default port: {}]", Protocol::Ftp.get_default_port().to_string()),
        long, required = false, 
        num_args = 0..=1,
        require_equals = true,
        value_name = "PORT",
    )] pub ftp: Option<u32>,

    #[arg(
        default_missing_value = Protocol::Tftp.get_default_port().to_string(),
        help = format!("Start the TFTP server [default port: {}]", Protocol::Tftp.get_default_port().to_string()),
        long, required = false, 
        num_args = 0..=1,
        require_equals = true,
        value_name = "PORT",
    )] pub tftp: Option<u32>,

    #[arg(
        default_missing_value = Protocol::Dhcp.get_default_port().to_string(),
        help = format!("Start the DHCP server"),
        long, required = false,
        num_args = 0,
        // require_equals = true,
        value_name = "PORT",
    )] pub dhcp: Option<bool>,
}


