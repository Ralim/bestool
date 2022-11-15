mod beslink;
mod cmds;
mod serial_monitor;

use crate::cmds::{cmd_list_serial_ports, cmd_serial_port_monitor, cmd_write_image};
use clap::Parser;

// BES2300 programming utility for better cross platform support
// This is completely reverse engineered at this point; there ~may~ will be bugs

/* Key commands:
* - Write binary
* - List serial ports / Serial Monitor
* - Set commands in userdata partition
*/

#[derive(Parser, Debug)] // requires `derive` feature
#[command(name = "bestool")]
#[command(bin_name = "bestool")]
enum BesTool {
    ListSerialPorts(ListSerialPorts),
    SerialMonitor(SerialMonitor),
    WriteImage(WriteImage),
}

#[derive(clap::Args, Debug)]
#[command(author, version, about, long_about = None)]
struct ListSerialPorts {}

#[derive(clap::Args, Debug)]
#[command(author, version, about, long_about = None)]
struct SerialMonitor {
    serial_port_path: String,
    #[arg(short, long, default_value_t = 2000000)]
    baud_rate: u32,
}
#[derive(clap::Args, Debug)]
#[command(author, version, about, long_about = None)]
struct WriteImage {
    firmware_path: Option<std::path::PathBuf>,
    #[arg(short, long)]
    serial_port_path: String,
    #[arg(short, long, default_value_t = false)]
    monitor_after: bool,
    #[arg(short, long, default_value_t = 2000000)]
    baud_rate: u32,
}

fn main() {
    match BesTool::parse() {
        BesTool::ListSerialPorts(_) => cmd_list_serial_ports(),
        BesTool::SerialMonitor(args) => {
            cmd_serial_port_monitor(args.serial_port_path, args.baud_rate);
        }
        BesTool::WriteImage(args) => cmd_write_image(
            args.firmware_path.unwrap().to_str().unwrap().to_owned(),
            args.serial_port_path,
        ),
    }
}
