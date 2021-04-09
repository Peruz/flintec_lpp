use chrono::prelude::*;
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
        file_bad_datetimes,
    ) = parse_cli();

    println!(
        "read data from {} and save to {}",
        csvin.to_str().unwrap(),
        csvout.to_str().unwrap()
    );
    let mut tw = TimeLoad::from_csv(csvin);

    match file_bad_datetimes {
        Some(f) => {
            let vec_bad_dateimes = read_bad_datetimes(f);
            tw.rm_datetime(vec_bad_dateimes);
        }
        None => (),
    }

    let time_init: NaiveTime = NaiveTime::from_hms(9, 0, 0);
    let time_stop: NaiveTime = NaiveTime::from_hms(10, 30, 0);
    println!("removeing times between {} and {}", time_init, time_stop);
    tw.rm_timeinterval(time_init, time_stop);

    let mut ftw = tw.fillnan_missing_datetime();
    ftw.replacenan_invalid(999994.);
    ftw.check_range(min_load, max_load);
    if side != 0 {
        let mavg_window = make_window(2., 1., side);
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
