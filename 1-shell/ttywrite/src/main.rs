extern crate serial;
extern crate structopt;
extern crate xmodem;
#[macro_use] extern crate structopt_derive;

use std::path::PathBuf;
use std::time::Duration;

use structopt::StructOpt;
use serial::core::{CharSize, BaudRate, StopBits, FlowControl, SerialDevice, SerialPortSettings};
use xmodem::{Xmodem, Progress};

mod parsers;

use parsers::{parse_width, parse_stop_bits, parse_flow_control, parse_baud_rate};

#[derive(StructOpt, Debug)]
#[structopt(about = "Write to TTY using the XMODEM protocol by default.")]
struct Opt {
    #[structopt(short = "i", help = "Input file (defaults to stdin if not set)", parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(short = "b", long = "baud", parse(try_from_str = "parse_baud_rate"),
                help = "Set baud rate", default_value = "115200")]
    baud_rate: BaudRate,

    #[structopt(short = "t", long = "timeout", parse(try_from_str),
                help = "Set timeout in seconds", default_value = "10")]
    timeout: u64,

    #[structopt(short = "w", long = "width", parse(try_from_str = "parse_width"),
                help = "Set data character width in bits", default_value = "8")]
    char_width: CharSize,

    #[structopt(help = "Path to TTY device", parse(from_os_str))]
    tty_path: PathBuf,

    #[structopt(short = "f", long = "flow-control", parse(try_from_str = "parse_flow_control"),
                help = "Enable flow control ('hardware' or 'software')", default_value = "none")]
    flow_control: FlowControl,

    #[structopt(short = "s", long = "stop-bits", parse(try_from_str = "parse_stop_bits"),
                help = "Set number of stop bits", default_value = "1")]
    stop_bits: StopBits,

    #[structopt(short = "r", long = "raw", help = "Disable XMODEM")]
    raw: bool,
}

fn main() {
    use std::fs::File;
    use std::io::{self, BufReader};

    let opt = Opt::from_args();
    let mut serial = serial::open(&opt.tty_path).expect("Path points to invalid TTY");

    let mut settings = serial.read_settings().expect("Failed to load settings");
    settings.set_baud_rate(opt.baud_rate).expect("Invalid baud rate");
    settings.set_char_size(opt.char_width);
    settings.set_flow_control(opt.flow_control);
    settings.set_stop_bits(opt.stop_bits);
    serial.write_settings(&settings).expect("Failed to apply serial settings");

    serial.set_timeout(Duration::from_secs(opt.timeout)).expect("Invalid timeout");

    let mut reader: Box<io::Read> = if let Some(path) = opt.input {
        let file = File::open(path).expect("Failed to open file");
        Box::new(BufReader::new(file))
    } else {
        Box::new(BufReader::new(io::stdin()))
    };

    if opt.raw {
        let bytes = io::copy(&mut reader, &mut serial).expect("Write failed");
        println!("Wrote {} bytes.", bytes);
    } else {
        let bytes = Xmodem::transmit_with_progress(reader, serial, |progress| {
            if let Progress::Packet(_) = progress {
                print!(".");
            } else if let Progress::Waiting = progress {
                println!("Ready");
            } else {
                assert!(false);
            }
        }).expect("Write failed");
        println!("");
        println!("Wrote {} bytes.", bytes);
    }
}
