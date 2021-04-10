use chrono::prelude::*;
use plotters::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
pub mod log;
pub mod plot;
pub mod process;

pub const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

pub const DT_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub const ERROR_STR_GENERAL: &str = "E+999999.";
pub const ERROR_STR_NONE: &str = "E+999998.";
pub const ERROR_STR_INVALID: &str = "E+999997.";
pub const ERROR_STR_SKIPPED: &str = "E+999996.";
pub const ERROR_STR_PARSE: &str = "E+999995.";

pub const ERROR_FLT_GENERAL: f64 = 999999.;
pub const ERROR_FLT_NONE: f64 = 999998.;
pub const ERROR_FLT_INVALID: f64 = 999997.;
pub const ERROR_FLT_SKIPPED: f64 = 999996.;
pub const ERROR_FLT_PARSE: f64 = 999995.;

/// The main struct for the load time series.
#[derive(Debug, Clone)]
pub struct TimeLoad {
    pub time: Vec<NaiveDateTime>,
    pub load: Vec<f64>,
}

impl TimeLoad {
    /// Initiate a new TimeLoad instance
    /// using the given capacity for the time and load vectors
    pub fn new(capacity: usize) -> TimeLoad {
        let time: Vec<NaiveDateTime> = Vec::with_capacity(capacity);
        let load: Vec<f64> = Vec::with_capacity(capacity);
        let timeload: TimeLoad = TimeLoad { time, load };
        timeload
    }

    /// Initiate a TimeLoad from csv
    /// setting load to NAN in case of load parsing errors,
    /// but panic for datatime errors.
    /// Do not check the continuity of the time series and presence of error flags,
    /// these are checked separately afterwards
    pub fn from_csv(fin: PathBuf) -> TimeLoad {
        let file = File::open(fin).unwrap();
        let buf = BufReader::new(file);
        let mut timeload = TimeLoad::new(10000 as usize);
        for l in buf.lines().skip(1) {
            let l_unwrap = match l {
                Ok(l_ok) => l_ok,
                Err(l_err) => {
                    println!("Err, could not read/unwrap line {}", l_err);
                    continue;
                }
            };
            let mut l_split = l_unwrap.split(',');
            let l_split_datetime = l_split.next().unwrap();
            let l_split_load = l_split.next().unwrap();
            match l_split_load.parse() {
                Ok(w) => {
                    timeload.load.push(w);
                    timeload
                        .time
                        .push(NaiveDateTime::parse_from_str(l_split_datetime, DT_FORMAT).unwrap());
                }
                _ => {
                    println!("invalid measurement found");
                    timeload.load.push(f64::NAN);
                    timeload
                        .time
                        .push(NaiveDateTime::parse_from_str(l_split_datetime, DT_FORMAT).unwrap());
                }
            }
        }
        timeload
    }

    /// Fill the datetime gaps with NAN to have continuous datetime.
    /// Take a reference to the read TimeLoad
    /// and return a new continuous TimeLoad.
    pub fn fillnan_missing_datetime(&self) -> TimeLoad {
        let mut timeload = TimeLoad::new(self.time.len());
        let datetime_first = self.time[0];
        let datetime_second = self.time[1];
        let delta_datetime = datetime_second - datetime_first;
        let mut dt_previous = datetime_first - delta_datetime;
        for (&dt, &w) in self.time.iter().zip(self.load.iter()) {
            if dt - dt_previous == delta_datetime {
                timeload.time.push(dt);
                timeload.load.push(w);
            } else if dt - dt_previous > delta_datetime {
                let mut expected_datetime = dt_previous + delta_datetime;
                while expected_datetime < dt {
                    timeload.time.push(expected_datetime);
                    timeload.load.push(f64::NAN);
                    expected_datetime += delta_datetime;
                }
                timeload.time.push(dt);
                timeload.load.push(w);
            }
            dt_previous = dt;
        }
        timeload
    }

    /// Drop all the datetime with NAN load, leaving a datetime gap.
    /// Take a reference and returns a new TimeLoad.
    pub fn removenan(&self) -> TimeLoad {
        let mut timeload = TimeLoad::new(self.time.len());
        for (&dt, &w) in self.time.iter().zip(self.load.iter()) {
            if w.is_nan() {
                continue;
            } else {
                timeload.time.push(dt);
                timeload.load.push(w);
            }
        }
        timeload
    }

    /// Consider all the values > max_value as invalid and replace them with NAN.
    /// Take a mutable reference to modify the TimeLoad in-place.
    pub fn replacenan_invalid(&mut self, max_value: f64) {
        for w in self.load.iter_mut() {
            if *w > max_value {
                println!("found invalid value: {}", w);
                *w = f64::NAN;
            }
        }
    }

    /// Set to NAN all the load values that are out of range.
    pub fn check_range(&mut self, max_load: f64, min_load: f64) {
        for w in self.load.iter_mut() {
            if (*w > max_load) | (*w < min_load) {
                println!(
                    "setting to NAN value out of range (min: {}, max {}): {}",
                    min_load, max_load, w
                );
                *w = f64::NAN;
            }
        }
    }

    /// Write the datetime and load columns as a csv at the given path.
    pub fn to_csv(self, fout: PathBuf) {
        let file = File::create(fout).unwrap();
        let mut buf = BufWriter::new(file);
        buf.write_all("datetime,load_kg\n".as_bytes()).unwrap();
        for (t, w) in self.time.iter().zip(self.load.iter()) {
            buf.write_all(format!("{},{}\n", t.to_string(), w).as_bytes())
                .unwrap();
        }
    }

    /// Plot the load time series to svg.
    pub fn plot_datetime(self, fout: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let (xmindt, xmaxdt): (NaiveDateTime, NaiveDateTime) = min_and_max(self.time.iter());
        let xspan: chrono::Duration = xmaxdt - xmindt;
        let xmargin: chrono::Duration = xspan / 20;
        let xmindt = xmindt - xmargin;
        let xmaxdt = xmaxdt + xmargin;
        let xminlocal = TimeZone::from_utc_datetime(&Utc, &xmindt);
        let xmaxlocal = TimeZone::from_utc_datetime(&Utc, &xmaxdt);
        let xfmt = suitable_xfmt(xspan);
        let (ymin, ymax) = min_and_max(self.load.iter().filter(|x| !x.is_nan()));
        let yspan = (ymax - ymin) / 10f64;
        let ymin = ymin - yspan;
        let ymax = ymax + yspan;
        let root = SVGBackend::new(&fout, (1600, 800)).into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .margin(20)
            .x_label_area_size(60)
            .y_label_area_size(100)
            .build_cartesian_2d(xminlocal..xmaxlocal, ymin..ymax)?;
        chart
            .configure_mesh()
            .light_line_style(&TRANSPARENT)
            .bold_line_style(RGBColor(150, 150, 150).stroke_width(2))
            .set_all_tick_mark_size(2)
            .label_style(("sans-serif", 24))
            .y_desc("load [kg]")
            .x_labels(14) // max number of labels
            .x_label_formatter(&|x: &DateTime<Utc>| x.format(xfmt).to_string())
            .y_label_formatter(&|x: &f64| format!("{:5}", x))
            .x_desc(format!("datetime [{}]", xfmt.replace("%", "")))
            .draw()?;

        let witer = &mut self.load[..].split(|x| x.is_nan());
        let titer = &mut self.time[..].into_iter();

        for wchunk in witer.into_iter() {
            if wchunk.len() == 0 {
                titer.next();
                continue;
            } else {
                let area = AreaSeries::new(
                    titer
                        .zip(wchunk)
                        .map(|(x, y)| (TimeZone::from_utc_datetime(&Utc, &x), *y)),
                    0.0,
                    &RED.mix(0.2),
                )
                .border_style(BLACK.stroke_width(1));
                chart.draw_series(area)?;
            }
        }
        Ok(())
    }

    pub fn rm_datetime(&mut self, mut bad_datetimes: Vec<NaiveDateTime>) {
        // Make sure it is sorted to then reverse it.
        // As we expect the vector to be already sorted,
        // sort() is the best algorithm, faster than sort_unstable(), see doc.
        bad_datetimes.sort();
        let len_bad_datetimes = bad_datetimes.len();
        let mut bad_indexes: Vec<usize> = Vec::with_capacity(len_bad_datetimes);
        for bdt in bad_datetimes.into_iter() {
            let index_bad = self.time.iter().position(|d| *d == bdt);
            match index_bad {
                Some(i) => {
                    println!("found and removed {} at index {}", bdt, i);
                    bad_indexes.push(i);
                }
                None => println!("{} not found", bdt),
            }
        }
        for bad_index in bad_indexes.into_iter().rev() {
            self.time.remove(bad_index);
            self.load.remove(bad_index);
        }
    }

    pub fn rm_timeinterval(&mut self, time_init: NaiveTime, time_stop: NaiveTime) {
        println!("intial len {}", self.time.len());
        let (keep_datetime, keep_load): (Vec<NaiveDateTime>, Vec<f64>) = self
            .time
            .iter()
            .zip(self.load.iter())
            .filter(|(&t, _)| (t.time() < time_init) | (t.time() > time_stop))
            .unzip();
        self.time = keep_datetime;
        self.load = keep_load;
        println!("final len {}", self.time.len());
    }
}

impl std::fmt::Display for TimeLoad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "datetime, load [kg]\n")?;
        for (t, w) in self.time.iter().zip(self.load.iter()) {
            write!(f, "{},{}\n", t.to_string(), w)?
        }
        Ok(())
    }
}

pub fn read_bad_datetimes(fin: PathBuf) -> Vec<NaiveDateTime> {
    let file = File::open(fin).unwrap();
    let buf = BufReader::new(file);
    let mut bad_datetimes: Vec<NaiveDateTime> = Vec::new();
    for l in buf.lines() {
        let l_unwrap = match l {
            Ok(l_ok) => l_ok,
            Err(l_err) => {
                println!("Err, could not read/unwrap line {}", l_err);
                continue;
            }
        };
        bad_datetimes.push(NaiveDateTime::parse_from_str(&l_unwrap, DT_FORMAT).unwrap());
    }
    return bad_datetimes;
}

pub fn min_and_max<'a, I, T>(mut s: I) -> (T, T)
where
    I: Iterator<Item = &'a T>,
    T: 'a + std::cmp::PartialOrd + Clone,
{
    let (mut min, mut max) = match s.next() {
        Some(v) => (v, v),
        None => panic!("could not iterate over slice"),
    };
    for es in s {
        if es > max {
            max = es
        } else if es < min {
            min = es
        }
    }
    return (min.clone(), max.clone());
}

pub fn suitable_xfmt(d: chrono::Duration) -> &'static str {
    let xfmt = if d > chrono::Duration::weeks(1) {
        "%y-%m-%d"
    } else if d > chrono::Duration::days(1) {
        "%m-%d %H"
    } else {
        "%d %H:%M"
    };
    return xfmt;
}

pub fn make_window(w_central: f64, w_side: f64, side: usize) -> Vec<f64> {
    let w_step = (w_central - w_side) / (side as f64);
    let up = (0..side + 1).map(|n| w_side + (n as f64 * w_step));
    let down = up.clone().rev().skip(1);
    let updown = up.chain(down).collect();
    updown
}

/// Roll the weighted moving window w over the data v,
/// also filling the NAN values with the weighted average when possible:
/// 1) sufficient number of data, i.e., number missing data under the window < max_missing_v.
/// 2) the window weight associated with the present data is sufficient, i.e.,
///     the percentage of missing weight is < than max_missing_wpct.
pub fn mavg(v: &[f64], w: &[f64], max_missing_v: usize, max_missing_wpct: f64) -> Vec<f64> {
    let len_v: i32 = v.len() as i32;
    let len_w: i32 = w.len() as i32;
    assert!(
        len_w < len_v,
        "length of moving average window > length of vector"
    );
    assert!(
        len_w % 2 == 1,
        "the moving average window has an even number of elements; \
        it should be odd to have a central element"
    );
    let side: i32 = (len_w - 1) / 2;
    let sum_all_w: f64 = w.iter().sum();
    let max_missing_w: f64 = sum_all_w / 100. * max_missing_wpct;
    let mut vout: Vec<f64> = Vec::with_capacity(len_v as usize);
    for i in 0..len_v {
        let mut missing_v = 0;
        let mut missing_w = 0.;
        let mut sum_ve_we = 0.;
        let mut sum_we = 0.;
        let mut ve: f64;
        let vl = i - side;
        let vr = i + side + 1;
        for (j, we) in (vl..vr).zip(w.iter()) {
            if (j < 0) || (j >= len_v) {
                missing_v += 1;
                missing_w += we;
            } else {
                ve = v[j as usize];
                if ve.is_nan() {
                    missing_v += 1;
                    missing_w += we;
                } else {
                    sum_ve_we += ve * we;
                    sum_we += we;
                }
            }
            if (missing_v > max_missing_v) || (missing_w > max_missing_w) {
                sum_ve_we = f64::NAN;
                println!(
                    "setting to NAN; {} missing data with limit {}, {} missing window weight with limit {}",
                    missing_v, max_missing_v, missing_w, max_missing_w,
                );
                break;
            }
        }
        vout.push(sum_ve_we / sum_we);
    }
    vout
}
