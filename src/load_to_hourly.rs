use super::VERSION;
use clap::{value_parser, Arg, Command};
use std::path::PathBuf;

/// Takes the CLI arguments that control the downsample of the load time series.
/// It is safe to unwrap clap cli_args.get_one when a default is given
/// because the default will be used when no argument is passed (i.e., it is always Some<T>).
pub fn parse_cli() -> (PathBuf, PathBuf) {

    let arg_csvin = Arg::new("input_csvfile")
        .help("name for the csv file")
        .short('f')
        .long("csvfile")
        .num_args(1)
        .value_parser(value_parser!(PathBuf))
        .default_value("loadcells.csv");

    let arg_csvout = Arg::new("output_csvfile")
        .help("name of the output csv file")
        .short('o')
        .long("csvfile")
        .value_parser(value_parser!(PathBuf))
        .num_args(1);

    let cli_args = Command::new("Flintec_downsample")
        .version(VERSION.unwrap_or("unknown"))
        .author("Luca Peruzzo")
        .about("cli app to downsample the load time series")
        .arg(arg_csvin)
        .arg(arg_csvout)
        .get_matches();

    let csvin: PathBuf = cli_args
        .get_one::<PathBuf>("input_csvfile")
        .unwrap()
        .to_owned();

    let csvout: PathBuf = match cli_args.get_one::<PathBuf>("output_csvfile") {
        Some(p) => p.to_owned(),
        None => csvin.with_file_name("hourly.csv"),
    };

    println!("read from {:?} and save to {:?}", csvin, csvout);

    return (csvin, csvout);
}
