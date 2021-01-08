use flintec_lp::plot::parse_cli;
use flintec_lp::TimeWeight;

fn main() {
    let (csvin, svgout) = parse_cli();
    println!(
        "read data from {} and plot to {}",
        csvin.to_str().unwrap(),
        svgout.to_str().unwrap()
    );
    let tw = TimeWeight::from_csv(csvin);
    // let nonnan_tw = tw.removenan();
    // nonnan_tw.plot_datetime(svgout).unwrap();
    tw.plot_datetime(svgout).unwrap();
}
