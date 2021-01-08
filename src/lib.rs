use chrono::prelude::*;
use plotters::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
pub mod log;
pub mod plot;
pub mod process;
pub mod realiterator;

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

/// The main struct for the load (weight) time series
#[derive(Debug, Clone)]
pub struct TimeWeight {
    pub time: Vec<NaiveDateTime>,
    pub weight: Vec<f64>,
}

impl TimeWeight {
    pub fn new(capacity: usize) -> TimeWeight {
        let time: Vec<NaiveDateTime> = Vec::with_capacity(capacity);
        let weight: Vec<f64> = Vec::with_capacity(capacity);
        let timeweight: TimeWeight = TimeWeight { time, weight };
        timeweight
    }

    /// Init a TimeWeight from csv
    /// setting weight to NAN in case of weight parsing errors,
    /// but panic for datatime errors.
    /// Do not check the continuity of the time series and presence of error flags,
    /// these are checked separately afterwards
    pub fn from_csv(fin: PathBuf) -> TimeWeight {
        let file = File::open(fin).unwrap();
        let buf = BufReader::new(file);
        let mut timeweight = TimeWeight::new(10000 as usize);
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
            let l_split_weight = l_split.next().unwrap();
            match l_split_weight.parse() {
                Ok(w) => {
                    timeweight.weight.push(w);
                    timeweight
                        .time
                        .push(NaiveDateTime::parse_from_str(l_split_datetime, DT_FORMAT).unwrap());
                }
                _ => {
                    println!("invalid measurement found");
                    timeweight.weight.push(f64::NAN);
                    timeweight
                        .time
                        .push(NaiveDateTime::parse_from_str(l_split_datetime, DT_FORMAT).unwrap());
                }
            }
        }
        timeweight
    }

    /// fill the datetime gaps with NAN to have continuous datetime
    /// takes a reference and returns a new TimeWeight
    pub fn fillnan_missing_datetime(&self) -> TimeWeight {
        let mut timeweight = TimeWeight::new(self.time.len());
        let datetime_first = self.time[0];
        let datetime_second = self.time[1];
        let delta_datetime = datetime_second - datetime_first;
        let mut dt_previous = datetime_first - delta_datetime;
        for (&dt, &w) in self.time.iter().zip(self.weight.iter()) {
            if dt - dt_previous == delta_datetime {
                timeweight.time.push(dt);
                timeweight.weight.push(w);
            } else if dt - dt_previous > delta_datetime {
                let mut expected_datetime = dt_previous + delta_datetime;
                while expected_datetime < dt {
                    timeweight.time.push(expected_datetime);
                    timeweight.weight.push(f64::NAN);
                    expected_datetime += delta_datetime;
                }
                timeweight.time.push(dt);
                timeweight.weight.push(w);
            }
            dt_previous = dt;
        }
        timeweight
    }

    /// drops all the datetime with NAN weight, leaving a datetime gap
    /// takes a reference and returns a new TimeWeight
    pub fn removenan(&self) -> TimeWeight {
        let mut timeweight = TimeWeight::new(self.time.len());
        for (&dt, &w) in self.time.iter().zip(self.weight.iter()) {
            if w.is_nan() {
                continue;
            } else {
                timeweight.time.push(dt);
                timeweight.weight.push(w);
            }
        }
        timeweight
    }

    /// consider all the values > max_value as invalid and replace with NAN
    /// takes a mutable reference to modify the TimeWeight in-place
    pub fn replacenan_invalid(&mut self, max_value: f64) {
        for w in self.weight.iter_mut() {
            if *w > max_value {
                println!("found invalid value: {}", w);
                *w = f64::NAN;
            }
        }
    }

    /// writes the datetime and weight columns as a csv at the given path
    pub fn to_csv(self, fout: PathBuf) {
        let file = File::create(fout).unwrap();
        let mut buf = BufWriter::new(file);
        buf.write_all("datetime,weight_kg\n".as_bytes()).unwrap();
        for (t, w) in self.time.iter().zip(self.weight.iter()) {
            buf.write_all(format!("{},{}\n", t.to_string(), w).as_bytes())
                .unwrap();
        }
    }

    /// plots the weight time series to svg
    pub fn plot_datetime(self, fout: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let (xmindt, xmaxdt): (NaiveDateTime, NaiveDateTime) = min_and_max(&self.time[..]);
        let xspan: chrono::Duration = xmaxdt - xmindt;
        let xmargin: chrono::Duration = xspan / 20;
        let xmindt = xmindt - xmargin;
        let xmaxdt = xmaxdt + xmargin;
        let xminlocal = TimeZone::from_utc_datetime(&Utc, &xmindt);
        let xmaxlocal = TimeZone::from_utc_datetime(&Utc, &xmaxdt);
        let xfmt = suitable_xfmt(xspan);
        let (ymin, ymax) = min_and_max(&self.weight[..]);
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
            .y_desc("weight [kg]")
            .x_labels(14) // max number of labels
            .x_label_formatter(&|x: &DateTime<Utc>| x.format(xfmt).to_string())
            .y_label_formatter(&|x: &f64| format!("{:5}", x))
            .x_desc(format!("datetime [{}]", xfmt.replace("%", "")))
            .draw()?;

        let witer = &mut self.weight[..].split(|x| x.is_nan());
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
        // other possible styles:
        // for wchunk in witer.into_iter() {
        //     if wchunk.len() == 0 {
        //         titer.next();
        //         continue
        //     } else {
        //         let line = LineSeries::new(
        //             titer
        //                 .zip(wchunk)
        //                 .map(|(x, y)| (TimeZone::from_utc_datetime(&Utc, &x), *y)),
        //             RGBColor(180, 10, 180).stroke_width(5),
        //         );
        //         chart.draw_series(line)?;
        //     }
        // }
        // let line = LineSeries::new(
        //     self.time
        //         .iter()
        //         .zip(self.weight.iter())
        //         .map(|(x, y)| (TimeZone::from_utc_datetime(&Utc, &x), *y)),
        //     RGBColor(100, 100, 100).stroke_width(5),
        // );
        // chart.draw_series(line)?;
        // let points = self.time.iter().zip(self.weight.iter()).map(|(x, y)| {
        //     Circle::new(
        //         (TimeZone::from_utc_datetime(&Utc, &x), *y),
        //         6,
        //         BLACK.filled(),
        //     )
        // });
        // chart.draw_series(points)?;
        Ok(())
    }
}

impl std::fmt::Display for TimeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "datetime, weight [kg]\n")?;
        for (t, w) in self.time.iter().zip(self.weight.iter()) {
            write!(f, "{},{}\n", t.to_string(), w)?
        }
        Ok(())
    }
}

pub fn min_and_max<T: std::cmp::PartialOrd + Copy>(s: &[T]) -> (T, T) {
    let mut self_iter = s.iter();
    let (mut min, mut max) = match self_iter.next() {
        Some(v) => (*v, *v),
        None => panic!("could not iterate over slice"),
    };
    for es in self_iter {
        if *es > max {
            max = *es
        }
        if *es < min {
            min = *es
        }
    }
    return (min, max);
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

pub fn make_window(a: f64, b: f64, s: usize) -> Vec<f64> {
    let window: Vec<f64> = if a == b {
        vec![b / (b * s as f64 * 2. + 1.); s * 2 + 1]
    } else {
        let step = (b - a) / (s as f64);
        let up = realiterator::FloatIterator::new_with_step(a, b + step, step);
        let down = realiterator::FloatIterator::new_with_step(b - step, a - step, step);
        let updown = up.chain(down);
        let updown_sum: f64 = updown.clone().sum();
        updown.into_iter().map(|v| v / updown_sum).collect()
    };
    let sum_check: f64 = window.iter().sum();
    assert!(
        0.98 < sum_check && sum_check < 1.02,
        "sum of moving average weights != 1 +- 0.02"
    );
    window
}

/// rolls the weighted moving window w over the data v
/// fills the NAN values with the weighted average when possible:
/// 1) sufficient number of data, i.e.,
///     number missing data under the window < max_missing_v
/// 2) the window weight associated with the present data is sufficient, i.e.,
///     the percentage of missing weight is < than max_missing_wpct
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
    let max_missing_w: f64 = sum_all_w / 100. * (100. - max_missing_wpct);
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
