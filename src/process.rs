use clap::{App, Arg};
use std::path::PathBuf;
use super::VERSION;

/// Takes the CLI arguments to set the processing parameters.
pub fn parse_cli() -> (PathBuf, PathBuf, usize, usize, f64, f64, f64) {
    let arg_csvin = Arg::with_name("input_csvfile")
        .help("name for the csv file")
        .short("f")
        .long("csvfile")
        .takes_value(true)
        .required(true);
    let arg_csvout = Arg::with_name("output_csvfile")
        .help("name of the output csv file")
        .short("o")
        .long("csvfile")
        .takes_value(true);
    let arg_side = Arg::with_name("side_length")
        .help("number of data points on each side for the moving average window")
        .short("s")
        .long("side")
        .takes_value(true)
        .default_value("60");
    let arg_mavg_values = Arg::with_name("mavg_values")
        .help("maximum missing percentage weight for the moving average")
        .short("n")
        .long("max_missing_values")
        .takes_value(true)
        .default_value("60");
    let arg_mavg_weight = Arg::with_name("mavg_weight")
        .help("maximum number of missing weights for the moving average")
        .short("w")
        .long("max_missing_weight")
        .takes_value(true)
        .default_value("50");
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

    let cli_args = App::new("smooth the weight time series")
        .version(VERSION.unwrap_or("unknown"))
        .author("Luca Peruzzo")
        .about("cli to smooth the weight time series")
        .arg(arg_csvin)
        .arg(arg_csvout)
        .arg(arg_side)
        .arg(arg_mavg_values)
        .arg(arg_mavg_weight)
        .arg(arg_max_load)
        .arg(arg_min_load)
        .get_matches();

    let csvin = PathBuf::from(cli_args.value_of("input_csvfile").unwrap());
    let csvout = match cli_args.value_of("output_csvfile") {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(csvin.to_str().unwrap().replace(".csv", "_processed.csv")),
    };
    let side = cli_args
        .value_of("side_length")
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
    return (
        csvin,
        csvout,
        side,
        mavg_max_missing_values,
        mavg_max_missing_pct_weight,
        max_load,
        min_load,
    );
}
