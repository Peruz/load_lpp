use super::VERSION;
use chrono::prelude::*;
use clap::{value_parser, Arg, Command};
use std::path::PathBuf;

/// Takes the CLI arguments to set the processing parameters.
pub fn parse_cli() -> (
    PathBuf,
    PathBuf,
    usize,
    usize,
    f64,
    f64,
    f64,
    bool,
    usize,
    f64,
    f64,
    f64,
    Option<PathBuf>,
    Option<(NaiveTime, NaiveTime)>,
    i32,
    bool,
) {
    let arg_in_raw_data = Arg::new("in_raw_data")
        .help("name for the input csv file with the data to process")
        .short('f')
        .long("inrawdata")
        .num_args(1)
        .value_parser(value_parser!(PathBuf))
        .required(true);
    let arg_out_proc_data = Arg::new("out_proc_data")
        .help("name for the output csv file with processed data")
        .short('o')
        .long("outprocdata")
        .value_parser(value_parser!(PathBuf))
        .num_args(1);
    let arg_mavg_side = Arg::new("mavg_side")
        .help("number of data points on each side for the moving average window")
        .short('s')
        .long("mavg_side")
        .num_args(1)
        .value_parser(value_parser!(usize))
        .default_value("2");
    let arg_mavg_max_missing_values = Arg::new("mavg_max_missing_values")
        .help("maximum missing number of values for the moving average")
        .long("mavg_max_missing_values")
        .num_args(1)
        .value_parser(value_parser!(usize))
        .default_value("3");
    let arg_mavg_max_missing_weight = Arg::new("mavg_max_missing_weight")
        .help("maximum percentage of missing weight for the moving average")
        .long("mavg_max_missing_weight")
        .num_args(1)
        .value_parser(value_parser!(f64))
        .default_value("80");
    let arg_mavg_central_weight = Arg::new("mavg_central_weight")
        .help("weight of the mavg central value")
        .long("mavg_central_weight")
        .num_args(1)
        .value_parser(value_parser!(f64))
        .default_value("3");
    let arg_mavg_side_weight = Arg::new("mavg_side_weight")
        .help("weight of the mavg ends")
        .long("mavg_side_weight")
        .num_args(1)
        .value_parser(value_parser!(f64))
        .default_value("1");
    let arg_anomaly_detect = Arg::new("anomaly_detect")
        .long("anomaly_detect")
        .num_args(0)
        .help("find and remove anomalous periods");
    let arg_anomaly_width = Arg::new("anomaly_width")
        .help("width of the anomaly detection window")
        .long("anomaly_width")
        .num_args(1)
        .value_parser(value_parser!(usize))
        .default_value("16");
    let arg_anomaly_iqr = Arg::new("anomaly_iqr")
        .long("anomaly_iqr")
        .num_args(1)
        .help("threshold for the anomaly detection as interquartile range")
        .value_parser(value_parser!(f64))
        .default_value("40");
    let arg_max_load = Arg::new("max_load")
        .help("maximum accepted load value")
        .long("max_load")
        .num_args(1)
        .value_parser(value_parser!(f64))
        .default_value("17000");
    let arg_min_load = Arg::new("min_load")
        .help("minimum accepted load value")
        .long("min_load")
        .num_args(1)
        .value_parser(value_parser!(f64))
        .default_value("13000");
    let arg_bad_datetimes = Arg::new("bad_datetimes")
        .help("name of the file with bad datetimes to be removed")
        .long("bad_datetimes")
        .num_args(0..2)
        .value_parser(value_parser!(PathBuf))
        .required(false);
    let arg_bad_time_interval = Arg::new("bad_time_interval")
        .help("daily time interval to be removed")
        .long("bad_time_interval")
        .num_args(2)
        .required(false);
    let arg_timezone = Arg::new("timezone")
        .help("timezone standard time relative to UTC")
        .allow_hyphen_values(true)
        .long("timezone")
        .num_args(1)
        .value_parser(value_parser!(i32))
        .default_value("-8");
    let arg_verbose = Arg::new("verbose")
        .help("print verbose information")
        .short('v')
        .long("verbose")
        .num_args(0..)
        .required(false);
    let cli_args = Command::new("Flintec_process")
        .version(VERSION.unwrap_or("unknown"))
        .author("Luca Peruzzo")
        .about("cli app to process the load time series: filter, refill, and smooth.")
        .arg(arg_in_raw_data)
        .arg(arg_out_proc_data)
        .arg(arg_mavg_side)
        .arg(arg_mavg_max_missing_values)
        .arg(arg_mavg_max_missing_weight)
        .arg(arg_mavg_central_weight)
        .arg(arg_mavg_side_weight)
        .arg(arg_anomaly_detect)
        .arg(arg_anomaly_width)
        .arg(arg_anomaly_iqr)
        .arg(arg_max_load)
        .arg(arg_min_load)
        .arg(arg_bad_datetimes)
        .arg(arg_bad_time_interval)
        .arg(arg_timezone)
        .arg(arg_verbose)
        .get_matches();
    let csvin = cli_args
        .get_one::<PathBuf>("in_raw_data")
        .unwrap()
        .to_owned();

    let csvout = match cli_args.get_one::<PathBuf>("out_proc_data") {
        Some(s) => s.to_owned(),
        None => {
            let new_fname = csvin
                .to_str()
                .expect("problems with file name encoding")
                .to_owned()
                + "_processed";
            csvin.with_file_name(&new_fname)
        }
    };
    let side = *cli_args.get_one::<usize>("mavg_side").unwrap();
    let mavg_max_missing_values = *cli_args
        .get_one::<usize>("mavg_max_missing_values")
        .unwrap();
    let mavg_max_missing_weight = *cli_args.get_one::<f64>("mavg_max_missing_weight").unwrap();
    let mavg_central_weight = *cli_args.get_one::<f64>("mavg_central_weight").unwrap();
    let mavg_side_weight = *cli_args.get_one::<f64>("mavg_side_weight").unwrap();
    let anomaly_detect = cli_args.contains_id("anomaly_detect");
    let anomaly_width = *cli_args.get_one::<usize>("anomaly_width").unwrap();
    let anomaly_iqr = *cli_args.get_one::<f64>("anomaly_iqr").unwrap();
    let max_load = *cli_args.get_one::<f64>("max_load").unwrap();
    let min_load = *cli_args.get_one::<f64>("min_load").unwrap();
    let bad_datetimes: Option<PathBuf> = cli_args
        .get_one::<PathBuf>("bad_datetimes")
        .map(|p| p.clone());
    let bad_time_interval: Option<(NaiveTime, NaiveTime)> =
        match cli_args.get_many::<String>("bad_time_interval") {
            Some(mut ti) => {
                let ts: String = ti.next().unwrap().to_string();
                let ts: NaiveTime = NaiveTime::parse_from_str(&ts, "%H:%M").unwrap();
                let te: String = ti.next().unwrap().to_string();
                let te: NaiveTime = NaiveTime::parse_from_str(&te, "%H:%M").unwrap();
                Some((ts, te))
            }
            None => None,
        };

    let timezone = *cli_args.get_one::<i32>("timezone").unwrap();
    let verbose: bool = cli_args.contains_id("verbose");

    return (
        csvin,
        csvout,
        side,
        mavg_max_missing_values,
        mavg_max_missing_weight,
        mavg_central_weight,
        mavg_side_weight,
        anomaly_detect,
        anomaly_width,
        anomaly_iqr,
        min_load,
        max_load,
        bad_datetimes,
        bad_time_interval,
        timezone,
        verbose,
    );
}
