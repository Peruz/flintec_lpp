use flintec_lp::make_window;
use flintec_lp::mavg;
use flintec_lp::process::parse_cli;
use flintec_lp::TimeWeight;

fn main() {
    let (csvin, csvout, side, mavg_max_missing_values, mavg_max_missing_pct_weight) = parse_cli();
    println!(
        "read data from {} and save to {}",
        csvin.to_str().unwrap(),
        csvout.to_str().unwrap()
    );
    let tw = TimeWeight::from_csv(csvin);
    let mut ftw = tw.fillnan_missing_datetime();
    ftw.replacenan_invalid(999994.);
    let mavg_window = make_window(5., 1., side);
    let smooth = mavg(
        &ftw.weight[..],
        &mavg_window,
        mavg_max_missing_values,
        mavg_max_missing_pct_weight,
    );
    ftw.weight = smooth;
    ftw.to_csv(csvout);
}
