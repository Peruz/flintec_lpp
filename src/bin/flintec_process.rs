use flintec_lpp::make_window;
use flintec_lpp::mavg;
use flintec_lpp::process::parse_cli;
use flintec_lpp::read_bad_datetimes;
use flintec_lpp::TimeLoad;

fn main() {
    let (
        csvin,
        csvout,
        side,
        mavg_max_missing_values,
        mavg_max_missing_pct_weight,
        min_load,
        max_load,
        bad_datetimes,
        bad_time_interval,
    ) = parse_cli();

    println!(
        "read data from {} and save to {}",
        csvin.to_str().unwrap(),
        csvout.to_str().unwrap()
    );
    let tw = TimeLoad::from_csv(csvin);

    tw.is_ordered();

    let mut ftw = tw.fill_missing_with_nan();

    ftw.is_ordered_and_continuous();

    if bad_datetimes.is_some() {
        let vec_bad_dateimes = read_bad_datetimes(bad_datetimes.unwrap());
        ftw.replace_bad_datetimes_with_nan(vec_bad_dateimes);
    }

    if bad_time_interval.is_some() {
        let t = bad_time_interval.unwrap();
        println!("removeing times between {} and {}", t.0, t.1);
        ftw.replace_bad_time_interval_with_nan(t.0, t.1);
    }

    ftw.replace_errors_with_nan(999994.);

    ftw.replace_outliers_with_nan(min_load, max_load);

    if side != 0 {
        let mavg_window = make_window(1., 1., side);
        let smooth = mavg(
            &ftw.load[..],
            &mavg_window,
            mavg_max_missing_values,
            mavg_max_missing_pct_weight,
        );
        ftw.load = smooth;
    }
    ftw.to_csv(csvout);
}
