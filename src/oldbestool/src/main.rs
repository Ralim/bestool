mod beslink;
mod cmds;
mod serial_monitor;
mod serial_port_opener;
use crate::cmds::{
    cmd_list_serial_ports, cmd_read_image, cmd_serial_port_monitor, cmd_write_image,
    cmd_write_image_then_monitor,
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
    WriteImageThenMonitor(WriteImageThenMonitor),
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
    #[arg(short, long, default_value_t = false)]
    wait: bool,
}
#[derive(clap::Args, Debug)]
#[command(author, version, about, long_about = None)]
struct WriteImage {
    firmware_path: std::path::PathBuf,
    #[arg(short, long)]
    port: String,
    #[arg(short, long, default_value_t = false)]
    wait: bool,
}
#[derive(clap::Args, Debug)]
#[command(author, version, about, long_about = None)]
struct WriteImageThenMonitor {
    firmware_path: std::path::PathBuf,
    #[arg(short, long)]
    port: String,
    #[arg(short, long, default_value_t = 2000000)]
    monitor_baud_rate: u32,
    #[arg(short, long, default_value_t = false)]
    wait: bool,
}
#[derive(clap::Args, Debug)]
#[command(author, version, about, long_about = None)]
struct ReadImage {
    firmware_path: std::path::PathBuf,
    #[arg(short, long)]
    port: String,
    #[arg(short, long, default_value_t = 1024*1024*4)] // default to full flash
    length: u32,
    #[arg(short, long, default_value_t = 0)] // default to start of flash
    offset: u32,
    #[arg(short, long, default_value_t = false)]
    wait: bool,
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
            cmd_serial_port_monitor(&args.serial_port_path, args.baud_rate, args.wait);
        }
        BesTool::WriteImage(args) => cmd_write_image(&args.firmware_path, &args.port, args.wait),
        BesTool::ReadImage(args) => cmd_read_image(
            &args.firmware_path,
            &args.port,
            args.offset as usize,
            args.length as usize,
            args.wait,
        ),
        BesTool::WriteImageThenMonitor(args) => cmd_write_image_then_monitor(
            &args.firmware_path,
            &args.port,
            args.monitor_baud_rate,
            args.wait,
        ),
    }
}
