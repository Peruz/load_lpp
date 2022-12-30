use super::VERSION;
use clap::{value_parser, Arg, Command};
use std::path::PathBuf;

/// Takes the CLI arguments that control the plotting of the load time series.
/// It is safe to unwrap clap cli_args.get_one when a default is given
/// because the default will be used when no argument is passed (i.e., it is always Some<T>).
/// svgout does not have a default because it is defined based on the csvin name
pub fn parse_cli() -> (PathBuf, PathBuf) {
    let arg_csvin = Arg::new("input_csvfile")
        .help("name for the csv file")
        .short('f')
        .long("csvfile")
        .num_args(1)
        .value_parser(value_parser!(PathBuf))
        .default_value("loadcells.csv");
    let arg_svgout = Arg::new("output_svgfile")
        .help("name of the output svg file")
        .short('o')
        .long("svgfile")
        .num_args(1);
    let cli_args = Command::new("Flintec_plot")
        .version(VERSION.unwrap_or("unknown"))
        .author("Luca Peruzzo")
        .about("cli app to plot the load time series")
        .arg(arg_csvin)
        .arg(arg_svgout)
        .get_matches();
    let csvin: PathBuf = cli_args
        .get_one::<PathBuf>("input_csvfile")
        .unwrap()
        .to_owned();
    println!("{:?}", csvin);
    let svgout = match cli_args.get_one::<PathBuf>("output_svgfile") {
        Some(p) => p.to_owned(),
        None => csvin.with_extension("svg"),
    };
    return (csvin, svgout);
}
