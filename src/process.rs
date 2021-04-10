use chrono::prelude::*;
use super::VERSION;
use clap::{App, Arg};
use std::path::PathBuf;

/// Takes the CLI arguments to set the processing parameters.
pub fn parse_cli() -> (
    PathBuf,
    PathBuf,
    usize,
    usize,
    f64,
    f64,
    f64,
    Option<PathBuf>,
    Option<(NaiveTime, NaiveTime)>,
) {
    let arg_in_raw_data = Arg::with_name("in_raw_data")
        .help("name for the input csv file with the data to process")
        .short("f")
        .long("inrawdata")
        .takes_value(true)
        .required(true);
    let arg_out_proc_data = Arg::with_name("out_proc_data")
        .help("name for the output csv file with processed data")
        .short("o")
        .long("outprocdata")
        .takes_value(true);
    let arg_side = Arg::with_name("mvavg_side")
        .help("number of data points on each side for the moving average window")
        .short("s")
        .long("side")
        .takes_value(true)
        .default_value("180");
    let arg_mavg_values = Arg::with_name("mavg_values")
        .help("maximum missing number of values for the moving average")
        .short("n")
        .long("mvavg_max_missing_values")
        .takes_value(true)
        .default_value("10000");
    let arg_mavg_weight = Arg::with_name("mavg_weight")
        .help("maximum percentage of missing weight for the moving average")
        .short("w")
        .long("mvavg_max_missing_weight")
        .takes_value(true)
        .default_value("80");
    let arg_max_load = Arg::with_name("max_load")
        .help("maximum accepted load value")
        .long("max_load")
        .takes_value(true)
        .default_value("15000");
    let arg_min_load = Arg::with_name("min_load")
        .help("minimum accepted load value")
        .long("min_load")
        .takes_value(true)
        .default_value("13000");
    let arg_bad_datetimes = Arg::with_name("bad_datetimes")
        .help("name of the file with bad datetimes to be removed")
        .short("b")
        .long("bad_datetimes")
        .takes_value(true)
        .required(false);
    let arg_bad_time_interval = Arg::with_name("bad_time_interval")
        .help("daily time interval to be removed")
        .short("t")
        .multiple(true)
        .long("bad_time_interval")
        .takes_value(true)
        .required(false);
    let cli_args = App::new("smooth the weight time series")
        .version(VERSION.unwrap_or("unknown"))
        .author("Luca Peruzzo")
        .about("cli to smooth the weight time series")
        .arg(arg_in_raw_data)
        .arg(arg_out_proc_data)
        .arg(arg_side)
        .arg(arg_mavg_values)
        .arg(arg_mavg_weight)
        .arg(arg_max_load)
        .arg(arg_min_load)
        .arg(arg_bad_datetimes)
        .arg(arg_bad_time_interval)
        .get_matches();

    let csvin = PathBuf::from(cli_args.value_of("in_raw_data").unwrap());
    let csvout = match cli_args.value_of("out_proc_data") {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(csvin.to_str().unwrap().replace(".csv", "_processed.csv")),
    };
    let side = cli_args
        .value_of("mvavg_side")
        .unwrap_or_default()
        .parse::<usize>()
        .unwrap();
    let mavg_max_missing_values = cli_args
        .value_of("mavg_values")
        .unwrap_or_default()
        .parse::<usize>()
        .unwrap();
    let mavg_max_missing_pct_weight = cli_args
        .value_of("mavg_weight")
        .unwrap_or_default()
        .parse::<f64>()
        .unwrap();
    let max_load = cli_args
        .value_of("max_load")
        .unwrap_or_default()
        .parse::<f64>()
        .unwrap();
    let min_load = cli_args
        .value_of("min_load")
        .unwrap_or_default()
        .parse::<f64>()
        .unwrap();
    let bad_datetimes: Option<PathBuf> = cli_args
        .value_of("bad_datetimes")
        .map(|f| PathBuf::from(f));
    let bad_time_interval: Option<(NaiveTime, NaiveTime)> = match cli_args.values_of("bad_time_interval") {
        Some(mut ti) => {
            let time_init = NaiveTime::parse_from_str(ti.next().unwrap(), "%H:%M").unwrap();
            let time_stop = NaiveTime::parse_from_str(ti.next().unwrap(), "%H:%M").unwrap();
            Some((time_init, time_stop))
        },
        None => None,
    };
    return (
        csvin,
        csvout,
        side,
        mavg_max_missing_values,
        mavg_max_missing_pct_weight,
        max_load,
        min_load,
        bad_datetimes,
        bad_time_interval,
    );
}
