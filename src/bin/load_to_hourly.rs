use load_lpp::load_to_hourly::parse_cli;
use load_lpp::TimeLoad;

fn main() {
    let (csvin, csvout) = parse_cli();
    println!(
        "read data from {} and plot to {}",
        csvin.to_str().unwrap(),
        csvout.to_str().unwrap()
    );
    let tw = TimeLoad::from_csv(csvin);
    let htw = tw.to_hourly();
    htw.to_csv(csvout)
}
