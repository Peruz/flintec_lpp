use flintec_lpp::plot::parse_cli;
use flintec_lpp::TimeLoad;

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
