use super::VERSION;
use chrono::prelude::*;
use clap::{App, Arg};

/// Takes the CLI arguments to control the logging application.
pub fn parse_cli_log() -> (String, String, u16, String, u32, u64, bool) {
    let arg_csvfile = Arg::with_name("csvfile")
        .help("name for the csv file")
        .short("o")
        .long("csvfile")
        .takes_value(true)
        .required(true)
        .default_value("loadcells.csv");
    let arg_ip = Arg::with_name("ip_address")
        .help("ip address for the telnet connection")
        .short("t")
        .long("ip")
        .takes_value(true)
        .required(true)
        .default_value("192.168.0.100");
    let arg_port = Arg::with_name("port")
        .help("port for the telnet connection")
        .short("p")
        .long("port")
        .takes_value(true)
        .default_value("23");
    let arg_tcmd = Arg::with_name("tcmd")
        .help("telnet command")
        .long_help("tcmd is automatically formatted, capitalization and enter; GetNet, GetAverage (128 readings over 1 sec)")
        .short("c")
        .long("tcmd")
        .required(true)
        .possible_values(&["gn", "ga", "GN", "GA"])
        .default_value("gn");
    let arg_minutes = Arg::with_name("minutes")
        .help("interlude and rounding for the reading times, in minutes")
        .short("m")
        .long("minutes")
        .required_unless("hours")
        .overrides_with("hours")
        .takes_value(true)
        .possible_values(&["1", "2", "3", "5", "10", "15", "20", "30", "60"])
        .default_value("2");
    let arg_hours = Arg::with_name("hours")
        .help("interlude and rounding for the reading times, in hours")
        .long("hours")
        .required_unless("minutes")
        .overrides_with("minutes")
        .takes_value(true)
        .possible_values(&["1", "2", "3", "6", "12", "24"]);
    let arg_delay = Arg::with_name("delay")
        .help("delay connection and logging, in minutes")
        .short("d")
        .long("delay")
        .required(true)
        .default_value("0");
    let arg_verbose = Arg::with_name("verbose")
        .help("print verbose information")
        .short("v")
        .long("verbose")
        .takes_value(false)
        .required(false);
    let cli_args = App::new("Flintec_log")
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
    let val_csvfile = String::from(cli_args.value_of("csvfile").unwrap_or_default());
    let val_ip = String::from(cli_args.value_of("ip_address").unwrap_or_default());
    let val_port = cli_args
        .value_of("port")
        .unwrap_or_default()
        .parse::<u16>()
        .unwrap();
    let val_tcmd = String::from(cli_args.value_of("tcmd").unwrap_or_default().to_uppercase());
    let val_delay = cli_args
        .value_of("delay")
        .unwrap_or_default()
        .parse::<u64>()
        .unwrap();
    let val_verbose: bool = cli_args.is_present("verbose");
    let val_minutes = cli_args.value_of("minutes");
    let val_hours = cli_args.value_of("hours");
    let val_interval: u32 = if val_hours.is_some() {
        val_hours.unwrap().parse::<u32>().unwrap() * 60 as u32
    } else {
        val_minutes.unwrap_or_default().parse::<u32>().unwrap()
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
    let first_local = Local.timestamp(first_sec, 0);
    first_local
}
