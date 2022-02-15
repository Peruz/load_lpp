#![feature(test)]
extern crate test;
use crate::utils::*;
use chrono::prelude::*;
use plotters::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub mod load_log_dad141;
pub mod load_plot;
pub mod load_process;
pub mod utils;

// constants
pub const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
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
    pub time: Vec<DateTime<FixedOffset>>,
    pub load: Vec<f64>,
}

impl TimeLoad {
    /// Initiate a new TimeLoad instance
    /// using the given capacity for the time and load vectors
    pub fn new(capacity: usize) -> TimeLoad {
        let time: Vec<DateTime<FixedOffset>> = Vec::with_capacity(capacity);
        let load: Vec<f64> = Vec::with_capacity(capacity);
        let timeload: TimeLoad = TimeLoad { time, load };
        timeload
    }

    /// Initiate a TimeLoad from csv
    /// setting load to NAN in case of load parsing errors,
    /// but panic for datatime errors.
    /// Do not check the continuity of the time series and presence of error flags,
    /// these are checked separately afterwards
    pub fn from_csv<P>(fin: P) -> TimeLoad
    where
        P: AsRef<Path>,
    {
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
            let parsed_datetime = match DateTime::parse_from_rfc3339(l_split_datetime) {
                Ok(parsed_datetime) => parsed_datetime,
                Err(e) => {
                    println!(
                        "Could not parse datetime: {}, error {}",
                        l_split_datetime, e
                    );
                    continue;
                }
            };
            timeload.time.push(parsed_datetime);
            match l_split_load.parse::<f64>() {
                Ok(parsed_load) => timeload.load.push(parsed_load),
                Err(e) => {
                    println!(
                        "Could not parse load: {}, at datetime {}. Error: {}",
                        l_split_load, parsed_datetime, e
                    );
                    timeload.load.push(f64::NAN);
                }
            }
        }
        timeload
    }

    pub fn is_ordered(&self) {
        self.time.windows(2).for_each(|w| {
            assert!(
                w[1] > w[0],
                "time series is not ordered: {} < {}",
                w[1],
                w[0]
            )
        });
    }

    pub fn is_ordered_and_continuous(&self) {
        self.time
            .windows(2)
            .map(|w| {
                assert!(
                    w[1] > w[0],
                    "time series is not ordered: {} < {}",
                    w[1],
                    w[0]
                );
                w[1] - w[0]
            })
            .reduce(|wp, wn| {
                assert_eq!(wp, wn, "time series is not continuous");
                wn
            });
    }

    /// Fill the datetime gaps with NAN to have continuous datetime.
    /// Take a reference to the read TimeLoad
    /// and return a new continuous TimeLoad.
    /// In fact, build a continuous datetime Vec and then match it with the load Vec?
    /// Use the minimum time interval in the data
    /// to determine the desired time step for the output.
    pub fn fill_missing_with_nan(&self) -> TimeLoad {
        let min_delta = self
            .time
            .windows(2)
            .map(|dtw| dtw[1] - dtw[0])
            .min()
            .unwrap();
        let mut timeload = TimeLoad::new(self.time.len());
        for (dtw, load) in self.time.windows(2).zip(self.load.iter()) {
            let mut current_dt: DateTime<FixedOffset> = dtw[0];
            timeload.time.push(current_dt);
            timeload.load.push(*load);
            while current_dt + min_delta < dtw[1] {
                current_dt = current_dt.checked_add_signed(min_delta).unwrap();
                timeload.time.push(current_dt);
                timeload.load.push(f64::NAN);
            }
        }
        timeload.time.push(self.time[self.time.len() - 1]);
        timeload.load.push(self.load[self.load.len() - 1]);
        timeload
    }

    /// Replace all values measured at the bad datetimes  nan.
    /// Need to be given as DateTime for correct and easier comparison.
    pub fn replace_bad_datetimes_with_nan(&mut self, bad_datetimes: Vec<DateTime<FixedOffset>>) {
        for bdt in bad_datetimes.into_iter() {
            match self.time.iter().position(|d| *d == bdt) {
                Some(i) => self.load[i] = f64::NAN,
                None => println!("could not find and exclude bad datetime {}", bdt),
            }
        }
    }

    /// Replace all values measured within the time interval with nan.
    /// Given in standard time, fixed offset for the chosen timezone.
    pub fn replace_bad_time_interval_with_nan(
        &mut self,
        time_init: NaiveTime,
        time_stop: NaiveTime,
    ) {
        self.time
            .iter()
            .zip(self.load.iter_mut())
            .for_each(|(t, l)| {
                if (t.time() > time_init) & (t.time() < time_stop) {
                    *l = f64::NAN;
                }
            });
    }

    /// Set to NAN all the load values that are out of the expected range.
    pub fn replace_outliers_with_nan(&mut self, min_load: f64, max_load: f64) {
        self.load.iter_mut().for_each(|l| {
            if (*l > max_load) | (*l < min_load) {
                println!(
                    "setting to NAN value out of range (min: {}, max {}): {}",
                    min_load, max_load, l
                );
                *l = f64::NAN;
            }
        });
    }

    /// Consider all the values > max_value as invalid and replace them with NAN.
    /// These high values are used for the errors.
    pub fn replace_errors_with_nan(&mut self, max_value: f64) {
        self.load.iter_mut().for_each(|l| {
            if *l > max_value {
                println!("found invalid value: {}", l);
                *l = f64::NAN;
            }
        });
    }

    /// Write the datetime and load columns to a csv file at the given path.
    /// Use RFC 3339 - ISO 8601 for datetime.
    pub fn to_csv<P>(self, fout: P)
    where
        P: AsRef<Path>,
    {
        let file = File::create(fout).unwrap();
        let mut buf = BufWriter::new(file);
        buf.write_all("datetime,load_kg\n".as_bytes()).unwrap();
        for (t, w) in self.time.iter().zip(self.load.iter()) {
            buf.write_all(format!("{},{}\n", t.to_rfc3339(), w).as_bytes())
                .unwrap();
        }
    }

    /// Plot the load time series to svg.
    pub fn plot_datetime<P>(&self, fout: P) -> Result<(), Box<dyn std::error::Error>>
    where
        P: AsRef<Path>,
    {
        let (xmin, xmax) = min_and_max(self.time.iter());
        let xspan: chrono::Duration = xmax - xmin;
        let xfmt = suitable_xfmt(xspan);
        let (ymin, ymax) = min_and_max(self.load.iter().filter(|x| !x.is_nan()));
        let yspan = (ymax - ymin) / 10f64;
        let ymin = ymin - yspan;
        let ymax = ymax + yspan;
        let root = SVGBackend::new(&fout, (1600, 800)).into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .margin(50)
            .x_label_area_size(40)
            .y_label_area_size(100)
            .build_cartesian_2d(xmin.clone()..xmax.clone(), ymin..ymax)?;
        chart
            .configure_mesh()
            .light_line_style(&TRANSPARENT)
            .bold_line_style(RGBColor(100, 100, 100).mix(0.5).stroke_width(2))
            .set_all_tick_mark_size(2)
            .label_style(("sans-serif", 20))
            .y_desc("load [kg]")
            .x_labels(16)
            .y_labels(25)
            .x_label_formatter(&|x| x.format(xfmt).to_string())
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
                let area =
                    AreaSeries::new(titer.zip(wchunk).map(|(x, y)| (*x, *y)), 0.0, &RED.mix(0.2))
                        .border_style(BLACK.stroke_width(1));
                chart.draw_series(area)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // run tests with:
    // cargo test -- --nocapture
    // to allow println! to stdout

    // DateTime<FixedOffset> stores:
    // self.datetime, which is the utc time
    // self.offset, accessed with self.offset.fix(), which is the fixedoffset
    // calling dt.naive_local() returns
    // self.datetime + self.offset.fix(), which is the standard time
    #[test]
    fn datetime_parsing_with_timezone() {
        let mut timezone: i32 = -8;
        timezone *= 60 * 60;
        let timezone_fixed_offset = FixedOffset::east(timezone);
        let dtstr = "2021-11-07T01:30:00-07:00";
        let dtiso = DateTime::parse_from_rfc3339(dtstr).unwrap();
        let dtfix = dtiso.with_timezone(&timezone_fixed_offset);
        println!(
            "datetime str {} parsed as {}, fixed {}",
            dtstr, dtiso, dtfix
        );
        let dtstr = "2021-11-07T01:30:00-08:00";
        let dtiso = DateTime::parse_from_rfc3339(dtstr).unwrap();
        let dtfix = dtiso.with_timezone(&timezone_fixed_offset);
        println!(
            "datetime str {} parsed as {}, fixed {}",
            dtstr, dtiso, dtfix
        );
    }

    fn f64eq_with_nan_eq(a: f64, b: f64) -> bool {
        (a.is_nan() && b.is_nan()) || (a == b)
    }

    fn f64vec_compare(va: &[f64], vb: &[f64]) -> bool {
        (va.len() == vb.len()) && va.iter().zip(vb).all(|(a, b)| f64eq_with_nan_eq(*a, *b))
    }

    #[test]
    fn test_all_steps() {
        let mut timezone: i32 = -8;
        timezone *= 60 * 60;
        let timezone_fixed_offset = FixedOffset::east(timezone);
        let mut tl = TimeLoad::from_csv(String::from("./test/datetime.csv"));
        tl.time
            .iter_mut()
            .for_each(|t| *t = t.with_timezone(&timezone_fixed_offset));
        println!("{}", tl);
        tl.is_ordered();
        let mut ctl = tl.fill_missing_with_nan();
        println!("{}", ctl);
        ctl.is_ordered_and_continuous();
        let bad = read_bad_datetimes("./test/bad_datetimes.csv");
        ctl.replace_bad_datetimes_with_nan(bad);
        println!("{}", ctl);
        let time_init = NaiveTime::parse_from_str("01:02", "%H:%M").unwrap();
        let time_stop = NaiveTime::parse_from_str("01:05", "%H:%M").unwrap();
        ctl.replace_bad_time_interval_with_nan(time_init, time_stop);
        println!("{}", ctl);
        ctl.replace_errors_with_nan(99995.);
        println!("{}", ctl);
        ctl.replace_outliers_with_nan(10000., 18000.);
        println!("{}", ctl);
        let mavg_window = make_window(3., 1., 2usize);
        let smooth = mavg(&ctl.load[..], &mavg_window, 3 as usize, 80.);
        println!("{:?}", smooth);
        ctl.load = smooth;
        ctl.plot_datetime("./test/test_all_steps.svg").unwrap();
        println!("{}", ctl);
        ctl.to_csv("./test/datetime_processed.csv");
        println!("saved to ./test/test_all_steps_processed.csv");
    }

    #[test]
    fn test_all_steps_parallel() {
        let mut timezone: i32 = -8;
        timezone *= 60 * 60;
        let timezone_fixed_offset = FixedOffset::east(timezone);
        let mut tl = TimeLoad::from_csv(String::from("./test/datetime_for_parallel.csv"));
        tl.time
            .iter_mut()
            .for_each(|t| *t = t.with_timezone(&timezone_fixed_offset));
        println!("{}", tl);
        tl.is_ordered();
        let mut ctl = tl.fill_missing_with_nan();
        println!("{}", ctl);
        ctl.is_ordered_and_continuous();
        let bad = read_bad_datetimes("./test/bad_datetimes.csv");
        ctl.replace_bad_datetimes_with_nan(bad);
        println!("{}", ctl);
        let time_init = NaiveTime::parse_from_str("01:02", "%H:%M").unwrap();
        let time_stop = NaiveTime::parse_from_str("01:05", "%H:%M").unwrap();
        ctl.replace_bad_time_interval_with_nan(time_init, time_stop);
        println!("{}", ctl);
        ctl.replace_errors_with_nan(99995.);
        println!("{}", ctl);
        ctl.replace_outliers_with_nan(10000., 18000.);
        println!("{}", ctl);
        let mavg_window = make_window(3., 1., 2usize);
        let smooth = mavg_parallel_fold(&ctl.load[..], &mavg_window);
        let correct_smooth = vec![
            f64::NAN,
            f64::NAN,
            13003.0,
            13004.0,
            13005.0,
            13006.0,
            13007.0,
            13008.0,
            13008.888888888889,
            13009.666666666666,
            13010.333333333334,
            13011.111111111111,
            13012.0,
            13013.0,
            13014.0,
            13015.0,
            13016.0,
            13017.0,
            13018.0,
            f64::NAN,
            f64::NAN,
        ];
        assert! {f64vec_compare(&smooth, &correct_smooth)};
        ctl.load = smooth;
        ctl.to_csv("./test/test_all_steps_parallel.csv");
        println!("saved to ./test/datetime_processed_parallel.csv");
    }

    #[test]
    fn test_logapp_datetime() {
        let dtnow: DateTime<Local> = Local::now();
        println!(
            "local time is {}",
            dtnow.to_rfc3339_opts(SecondsFormat::Secs, false)
        );
        let local_offset = dtnow.offset();
        println!("local offet is {}", local_offset);
    }

    #[test]
    fn test_find_anomaly_homogeneous() {
        let a = [5.0f64; 15];
        let expected: Vec<f64> = Vec::new();
        let anomalies = find_anomalis(&a, 7usize, 6usize, 5.0f64);
        assert!(anomalies == expected);
    }

    #[test]
    fn test_find_anomaly_linear() {
        let v: Vec<f64> = (1..15).map(|n| n as f64).collect();
        let expected: Vec<f64> = Vec::new();
        let anomalies = find_anomalis(&v, 7usize, 6usize, 5.0f64);
        assert!(anomalies == expected);
    }
}

#[bench]
fn bench_mavg_parallel_simd(b: &mut test::Bencher) {
    let v = vec![1000.; 1E+5 as usize];
    let w = make_window(3., 1., 180 as usize);
    b.iter(|| {
        mavg_parallel_simd(&v, &w);
    });
}

#[bench]
fn bench_mavg_parallel_fold(b: &mut test::Bencher) {
    let v = vec![1000.; 1E+5 as usize];
    let w = make_window(3., 1., 180 as usize);
    b.iter(|| {
        mavg_parallel_fold(&v, &w);
    });
}

#[bench]
fn bench_mavg(b: &mut test::Bencher) {
    let v = vec![1000.; 1E+5 as usize];
    let w = make_window(3., 1., 180 as usize);
    b.iter(|| {
        mavg(&v, &w, 1usize, 1f64);
    });
}
