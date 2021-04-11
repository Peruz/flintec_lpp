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
    let mut tw = TimeLoad::from_csv(csvin);

    match bad_datetimes {
        Some(f) => {
            let vec_bad_dateimes = read_bad_datetimes(f);
            tw.rm_datetime(vec_bad_dateimes);
        }
        None => (),
    }

    match bad_time_interval {
        Some(t) => {
            println!("removeing times between {} and {}", t.0, t.1);
            tw.rm_timeinterval(t.0, t.1);
        },
        None => (),
    }

    let mut ftw = tw.fillnan_missing_datetime_robust();

    ftw.replacenan_invalid_with_nan(999994.);

    ftw.set_outliers_to_nan(min_load, max_load);

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
