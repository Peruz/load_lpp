use super::VERSION;
use chrono::prelude::*;
use clap::{Arg, Command};

/// Takes the CLI arguments to control the logging application.
/// Use hours (times 60) if given, otherwise use minutes.
/// When both are given, the last given is considered (overriding behavior).
/// Minutes and hours can be safely unwrapped, the list of possible values is enforced by clap itself.
pub fn parse_cli_log() -> (String, String, u16, String, u32, u64, bool) {
    let arg_csvfile = Arg::new("csvfile")
        .help("name for the csv file")
        .short('o')
        .long("csvfile")
        .num_args(1)
        .default_value("loadcells.csv");
    let arg_ip = Arg::new("ip_address")
        .help("ip address for the telnet connection")
        .short('t')
        .long("ip")
        .num_args(1)
        .default_value("192.168.0.100");
    let arg_port = Arg::new("port")
        .help("port for the telnet connection")
        .short('p')
        .long("port")
        .num_args(1)
        .default_value("23");
    let arg_tcmd = Arg::new("tcmd")
        .help("telnet command")
        .long_help("tcmd is automatically formatted, capitalization and enter; GetNet, GetAverage (128 readings over 1 sec)")
        .short('c')
        .long("tcmd")
        .num_args(1)
        .value_parser(["gn", "ga", "GN", "GA"])
        .default_value("gn");
    let arg_minutes = Arg::new("minutes")
        .help("interlude and rounding for the reading times, in minutes")
        .short('m')
        .long("minutes")
        .overrides_with("hours")
        .num_args(1)
        .value_parser(["1", "2", "3", "5", "10", "15", "20", "30", "60"])
        .default_value("2");
    let arg_hours = Arg::new("hours")
        .help("interlude and rounding for the reading times, in hours")
        .long("hours")
        .overrides_with("minutes")
        .num_args(1)
        .value_parser(["1", "2", "3", "6", "12", "24"]);
    let arg_delay = Arg::new("delay")
        .help("delay connection and logging, in minutes")
        .short('d')
        .long("delay")
        .default_value("0");
    let arg_verbose = Arg::new("verbose")
        .help("print verbose information")
        .short('v')
        .long("verbose")
        .num_args(0..)
        .required(false);
    let cli_args = Command::new("Flintec_log")
        .version(VERSION.unwrap_or("unknown"))
        .author("Luca Peruzzo")
        .about("cli app to log the load cells")
        .arg(arg_csvfile)
        .arg(arg_minutes)
        .arg(arg_hours)
        .arg(arg_tcmd)
        .arg(arg_delay)
        .arg(arg_verbose)
        .arg(arg_ip)
        .arg(arg_port)
        .get_matches();
    let val_csvfile = cli_args.get_one::<String>("csvfile").unwrap().to_owned();
    let val_ip = cli_args.get_one::<String>("ip_address").unwrap().to_owned();
    let val_port = cli_args
        .get_one::<String>("port")
        .unwrap()
        .to_owned()
        .parse::<u16>()
        .expect("invalid port argument, could not parse string to u16");
    let val_tcmd = cli_args.get_one::<String>("tcmd").unwrap().to_uppercase();
    let val_delay = cli_args
        .get_one::<String>("delay")
        .unwrap()
        .to_owned()
        .parse::<u64>()
        .expect("invalid delay argument, could not parse string to u64");
    let val_verbose: bool = cli_args.contains_id("verbose");
    let val_interval: u32 = match cli_args.get_one::<String>("hours") {
        Some(s) => s.to_owned().parse::<u32>().unwrap() * 60 as u32,
        None => cli_args
            .get_one::<String>("minutes")
            .unwrap()
            .to_owned()
            .parse::<u32>()
            .unwrap(),
    };

    return (
        val_csvfile,
        val_ip,
        val_port,
        val_tcmd,
        val_interval,
        val_delay,
        val_verbose,
    );
}

pub fn prepare_csvfile(file: &str) -> std::fs::File {
    if std::path::Path::new(&file).exists() {
        println!("csvfile {} already exists, values will be appended", file);
    } else {
        match std::fs::write(&file, "datetime,load_kg,raw_reading\n") {
            Ok(_) => println!("initiated csvfile {}", file),
            Err(e) => panic!("could not initiate csvfile {}, error: {}", file, e),
        }
    }
    let file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&file)
        .unwrap();
    return file;
}

pub fn chrono_first_rounded(
    datetime: DateTime<Local>,
    rounding: chrono::Duration,
) -> DateTime<Local> {
    let offset: i64 = datetime.offset().local_minus_utc().into();
    let local_sec = datetime.timestamp() + offset;
    let rounding_sec = rounding.num_seconds();
    let first_sec = rounding_sec * ((local_sec + rounding_sec) / rounding_sec) - offset;
    let first_local = Local.timestamp_opt(first_sec, 0).unwrap();
    first_local
}
