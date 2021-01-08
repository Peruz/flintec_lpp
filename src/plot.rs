use clap::{App, Arg};
use std::path::PathBuf;

pub fn parse_cli() -> (PathBuf, PathBuf) {
    let arg_csvin = Arg::with_name("input_csvfile")
        .help("name for the csv file")
        .short("f")
        .long("csvfile")
        .takes_value(true)
        .required(true)
        .default_value("loadcells.csv");
    let arg_svgout = Arg::with_name("output_svgfile")
        .help("name of the output svg file")
        .short("o")
        .long("svgfile")
        .takes_value(true);
    let cli_args = App::new("plot load cells data")
        .version("0.1.0")
        .author("Luca Peruzzo")
        .about("simple cli app to plot the load time series")
        .arg(arg_csvin)
        .arg(arg_svgout)
        .get_matches();
    let csvin = PathBuf::from(cli_args.value_of("input_csvfile").unwrap_or_default());
    let svgout = match cli_args.value_of("output_svgfile") {
        Some(p) => PathBuf::from(p),
        None => {
            let mut svgout = csvin.clone();
            svgout.set_extension("svg");
            svgout
        }
    };
    return (csvin, svgout);
}
