use clap::{App, Arg};
use std::path::PathBuf;

pub fn parse_cli() -> (PathBuf, PathBuf, usize, usize, f64) {
    let arg_csvin = Arg::with_name("input_csvfile")
        .help("name for the csv file")
        .short("f")
        .long("csvfile")
        .takes_value(true)
        .required(true);
    let arg_csvout = Arg::with_name("output_csvfile")
        .help("name of the output csv file")
        .short("o")
        .long("csvfile")
        .takes_value(true);
    let arg_side = Arg::with_name("side_length")
        .help("number of data points on each side for the moving average window")
        .short("s")
        .long("side")
        .takes_value(true)
        .default_value("60");
    let arg_mavg_values = Arg::with_name("mavg_values")
        .help("maximum number of missing weights for the moving average")
        .short("n")
        .long("max_missing_values")
        .takes_value(true)
        .default_value("60");
    let arg_mavg_weight = Arg::with_name("mavg_weight")
        .help("maximum number of missing weights for the moving average")
        .short("w")
        .long("max_missing_values")
        .takes_value(true)
        .default_value("50");
    let cli_args = App::new("smooth the weight time series")
        .version("0.1.0")
        .author("Luca Peruzzo")
        .about("cli to smooth the weight time series")
        .arg(arg_csvin)
        .arg(arg_csvout)
        .arg(arg_side)
        .arg(arg_mavg_values)
        .arg(arg_mavg_weight)
        .get_matches();
    let csvin = PathBuf::from(cli_args.value_of("input_csvfile").unwrap());
    let csvout = match cli_args.value_of("output_csvfile") {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(csvin.to_str().unwrap().replace(".csv", "_processed.csv")),
    };
    let side = cli_args
        .value_of("side_length")
        .unwrap_or_default()
        .parse::<usize>()
        .unwrap();
    let mavg_max_missing_values = cli_args
        .value_of("mavg_values")
        .unwrap_or_default()
        .parse::<usize>()
        .unwrap();
    let mavg_max_missing_pct_weight = cli_args
        .value_of("mavg_weight")
        .unwrap_or_default()
        .parse::<f64>()
        .unwrap();
    return (
        csvin,
        csvout,
        side,
        mavg_max_missing_values,
        mavg_max_missing_pct_weight,
    );
}