use super::VERSION;
use chrono::prelude::*;
use clap::{Command, Arg};
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
) {
    let arg_in_raw_data = Arg::new("in_raw_data")
        .help("name for the input csv file with the data to process")
        .short('f')
        .long("inrawdata")
        .takes_value(true)
        .required(true);
    let arg_out_proc_data = Arg::new("out_proc_data")
        .help("name for the output csv file with processed data")
        .short('o')
        .long("outprocdata")
        .takes_value(true);
    let arg_mavg_side = Arg::new("mavg_side")
        .help("number of data points on each side for the moving average window")
        .short('s')
        .long("mavg_side")
        .takes_value(true)
        .default_value("2");
    let arg_mavg_max_missing_values = Arg::new("mavg_max_missing_values")
        .help("maximum missing number of values for the moving average")
        .long("mavg_max_missing_values")
        .takes_value(true)
        .default_value("3");
    let arg_mavg_max_missing_weight = Arg::new("mavg_max_missing_weight")
        .help("maximum percentage of missing weight for the moving average")
        .long("mavg_max_missing_weight")
        .takes_value(true)
        .default_value("80");
    let arg_mavg_central_weight = Arg::new("mavg_central_weight")
        .help("weight of the mavg central value")
        .long("mavg_central_weight")
        .takes_value(true)
        .default_value("3");
    let arg_mavg_side_weight = Arg::new("mavg_side_weight")
        .help("weight of the mavg ends")
        .long("mavg_side_weight")
        .takes_value(true)
        .default_value("1");
    let arg_anomaly_detect = Arg::new("anomaly_detect")
        .long("anomaly_detect")
        .takes_value(false)
        .help("find and remove anomalous periods");
    let arg_anomaly_width = Arg::new("anomaly_width")
        .long("anomaly_width")
        .takes_value(true)
        .help("width of the anomaly detection window")
        .default_value("16");
    let arg_anomaly_iqr = Arg::new("anomaly_iqr")
        .long("anomaly_iqr")
        .takes_value(true)
        .help("threshold for the anomaly detection as interquartile range")
        .default_value("40");
    let arg_max_load = Arg::new("max_load")
        .help("maximum accepted load value")
        .long("max_load")
        .takes_value(true)
        .default_value("17000");
    let arg_min_load = Arg::new("min_load")
        .help("minimum accepted load value")
        .long("min_load")
        .takes_value(true)
        .default_value("13000");
    let arg_bad_datetimes = Arg::new("bad_datetimes")
        .help("name of the file with bad datetimes to be removed")
        .long("bad_datetimes")
        .takes_value(true)
        .required(false);
    let arg_bad_time_interval = Arg::new("bad_time_interval")
        .help("daily time interval to be removed")
        .multiple_values(true)
        .long("bad_time_interval")
        .takes_value(true)
        .required(false);
    let ard_timezone = Arg::new("timezone")
        .help("timezone standard time relative to UTC")
        .allow_hyphen_values(true)
        .long("timezone")
        .takes_value(true)
        .default_value("-8");
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
        .arg(ard_timezone)
        .get_matches();

    let csvin = PathBuf::from(cli_args.value_of("in_raw_data").unwrap());
    let csvout = match cli_args.value_of("out_proc_data") {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(csvin.to_str().unwrap().replace(".csv", "_processed.csv")),
    };
    let side = cli_args
        .value_of("mavg_side")
        .unwrap_or_default()
        .parse::<usize>()
        .unwrap();
    let mavg_max_missing_values = cli_args
        .value_of("mavg_max_missing_values")
        .unwrap_or_default()
        .parse::<usize>()
        .unwrap();
    let mavg_max_missing_weight = cli_args
        .value_of("mavg_max_missing_weight")
        .unwrap_or_default()
        .parse::<f64>()
        .unwrap();
    let mavg_central_weight = cli_args
        .value_of("mavg_central_weight")
        .unwrap_or_default()
        .parse::<f64>()
        .unwrap();
    let mavg_side_weight = cli_args
        .value_of("mavg_side_weight")
        .unwrap_or_default()
        .parse::<f64>()
        .unwrap();
    let anomaly_detect = cli_args
        .is_present("anomaly_detect");
    let anomaly_width = cli_args
        .value_of("anomaly_width")
        .unwrap_or_default()
        .parse::<usize>()
        .unwrap();
    let anomaly_iqr = cli_args
        .value_of("anomaly_iqr")
        .unwrap_or_default()
        .parse::<f64>()
        .unwrap();
    let max_load = cli_args
        .value_of("max_load")
        .unwrap_or_default()
        .parse::<f64>()
        .unwrap();
    let min_load = cli_args
        .value_of("min_load")
        .unwrap_or_default()
        .parse::<f64>()
        .unwrap();
    let bad_datetimes: Option<PathBuf> =
        cli_args.value_of("bad_datetimes").map(|f| PathBuf::from(f));
    let bad_time_interval: Option<(NaiveTime, NaiveTime)> =
        match cli_args.values_of("bad_time_interval") {
            Some(mut ti) => {
                let time_init = NaiveTime::parse_from_str(ti.next().unwrap(), "%H:%M").unwrap();
                let time_stop = NaiveTime::parse_from_str(ti.next().unwrap(), "%H:%M").unwrap();
                Some((time_init, time_stop))
            }
            None => None,
        };
    let timezone = cli_args
        .value_of("timezone")
        .unwrap_or_default()
        .parse::<i32>()
        .unwrap();

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
    );
}
