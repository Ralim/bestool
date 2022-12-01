mod beslink;
mod cmds;
mod serial_monitor;
use crate::cmds::{
    cmd_list_serial_ports, cmd_read_image, cmd_serial_port_monitor, cmd_write_image,
};
use clap::Parser;
use tracing::Level;

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
    ReadImage(ReadImage),
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
    port: String,
    #[arg(short, long, default_value_t = false)]
    monitor_after: bool,
    #[arg(short, long, default_value_t = 2000000)]
    baud_rate: u32,
}
#[derive(clap::Args, Debug)]
#[command(author, version, about, long_about = None)]
struct ReadImage {
    firmware_path: Option<std::path::PathBuf>,
    #[arg(short, long)]
    port: String,
    #[arg(short, long, default_value_t = 1024*1024*4)] // default to full flash
    length: u32,
    #[arg(short, long, default_value_t = 0)] // default to start of flash
    offset: u32,
}

fn main() {
    // install global subscriber configured based on RUST_LOG envvar.
    let subscriber = tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    match BesTool::parse() {
        BesTool::ListSerialPorts(_) => cmd_list_serial_ports(),
        BesTool::SerialMonitor(args) => {
            cmd_serial_port_monitor(args.serial_port_path, args.baud_rate);
        }
        BesTool::WriteImage(args) => cmd_write_image(
            args.firmware_path.unwrap().to_str().unwrap().to_owned(),
            args.port,
        ),
        BesTool::ReadImage(args) => cmd_read_image(
            args.firmware_path.unwrap().to_str().unwrap().to_owned(),
            args.port,
            args.offset as usize,
            args.length as usize,
        ),
    }
}
