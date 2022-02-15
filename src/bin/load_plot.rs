use load_lpp::load_plot::parse_cli;
use load_lpp::TimeLoad;

fn main() {
    let (csvin, svgout) = parse_cli();
    println!(
        "read data from {} and plot to {}",
        csvin.to_str().unwrap(),
        svgout.to_str().unwrap()
    );
    let tw = TimeLoad::from_csv(csvin);
    tw.plot_datetime(svgout).unwrap();
}
