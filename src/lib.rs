#![feature(test)]
#![feature(slice_partition_dedup)]
extern crate test;
pub use crate::utils::*;
use chrono::prelude::*;
use plotters::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub mod load_log_dad141;
pub mod load_plot;
pub mod load_process;
pub mod utils;

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
    /// these are checked separately afterwards.
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

    // Assert that the time series is ordered.
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

    // Assert that the time series is ordered and continuous.
    pub fn is_ordered_and_continuous(&self) {
        self.time
            .windows(2)
            .map(|w| {
                assert!( w[1] > w[0], "time series is not ordered: {} < {}", w[1], w[0]);
                w[1] - w[0]
            })
            .reduce(|wp, wn| {
                assert_eq!(wp, wn, "time series is not continuous");
                wn
            });
    }

    /// Fill the datetime gaps with NANs to have continuous datetime.
    /// Take a reference to the read TimeLoad and return a new continuous TimeLoad.
    /// Heuristically use the minimum time interval in the data to determine the desired time step for the output.
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

    /// Set to NAN the load values corresponsiding to the input bad datetimes.
    pub fn replace_bad_datetimes_with_nan(&mut self, bad_datetimes: Vec<DateTime<FixedOffset>>) {
        for bdt in bad_datetimes.into_iter() {
            match self.time.iter().position(|d| *d == bdt) {
                Some(i) => self.load[i] = f64::NAN,
                None => println!("could not find and exclude bad datetime {}", bdt),
            }
        }
    }

    /// Replace all values measured within the time interval with NANs.
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
    /// These high values are reserved for the errors.
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

impl std::fmt::Display for TimeLoad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "datetime, load [kg]\n")?;
        for (t, w) in self.time.iter().zip(self.load.iter()) {
            write!(f, "{},{}\n", t.to_rfc3339(), w)?
        }
        Ok(())
    }
}

// use crate::utils::compare_vecf64;
// Run the tests with:
// cargo test -- --nocapture
// to allow println!(...) to stdout.
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    // Test the correct correction for the daylight saving,
    // needed for long term monitorings
    fn datetime_parsing_with_timezone() {
        let timezone_hours: i32 = -8;
        let timezone = timezone_hours * 60 * 60;
        let timezone_fixed_offset = FixedOffset::east_opt(timezone).unwrap();

        // test DST
        let dtstr_dst = "2021-11-07T01:30:00-07:00";
        let dtiso_dst = DateTime::parse_from_rfc3339(dtstr_dst).unwrap();
        let dtfix_dst = dtiso_dst.with_timezone(&timezone_fixed_offset);
        println!(
            "datetime str {} parsed as {}, fixed {}",
            dtstr_dst, dtiso_dst, dtfix_dst
        );

        // test PST
        let dtstr_pst = "2021-11-07T01:30:00-08:00";
        let dtiso_pst = DateTime::parse_from_rfc3339(dtstr_pst).unwrap();
        let dtfix_pst = dtiso_pst.with_timezone(&timezone_fixed_offset);
        println!(
            "datetime str {} parsed as {}, fixed {}",
            dtstr_pst, dtiso_pst, dtfix_pst
        );

        // assert that DST goes 1 hour back
        let timediff = chrono::Duration::hours(1);
        assert!(dtfix_pst - timediff == dtfix_dst);
    }

    #[test]
    // Get the reading datetime with the correct offset
    fn test_get_current_datetime_offset() {
        let dtnow: DateTime<Local> = Local::now();
        println!(
            "local time is {}",
            dtnow.to_rfc3339_opts(SecondsFormat::Secs, false)
        );
        let local_offset = dtnow.offset();
        println!("local offet is {}", local_offset);
    }

    #[test]
    // Assert that a homogenous load time series gives no anomalies
    fn test_find_anomaly_homogeneous() {
        let a = [5.0f64; 15];
        let expected: Vec<f64> = Vec::new();
        let (_, anomalies_load) = find_anomalies(&a, 7usize, 6usize, 5.0f64);
        assert!(anomalies_load == expected);
    }

    #[test]
    // Assert that a linear load time series with small increment gives no anomalies
    fn test_find_anomaly_linear() {
        let v: Vec<f64> = (1..15).map(|n| n as f64).collect();
        let expected: Vec<f64> = Vec::new();
        let (_, anomalies_load) = find_anomalies(&v, 7usize, 6usize, 5.0f64);
        assert!(anomalies_load == expected);
    }

    #[test]
    // Assert that a NANs are correctly handled while finding anomalies
    fn test_find_anomaly_nans() {
        let mut v: Vec<f64> = (1..15).map(|n| n as f64).collect();
        v.iter_mut().enumerate().for_each(|(i, e)| {
            if i < 6usize {
                *e = f64::NAN
            }
        });
        let expected: Vec<f64> = Vec::new();
        let (_, anomalies_load) = find_anomalies(&v, 7usize, 6usize, 5.0f64);
        assert!(anomalies_load == expected);
    }

    #[test]
    // Assert that the anomalies are correctly identified by adding a big discontinuity
    fn test_find_anomaly_discontinuity() {
        let mut v: Vec<f64> = (1..15).map(|n| n as f64).collect();
        v.iter_mut().enumerate().for_each(|(i, e)| {
            if i < 8usize {
                *e = 20.
            }
        });
        let (_, anomalies_load) = find_anomalies(&v, 7usize, 6usize, 5.0f64);
        let expected: Vec<f64> = vec![20.0, 20.0, 20.0, 20.0, 9.0, 10.0, 11.0, 12.0, 13.0];
        assert!(anomalies_load == expected);
    }

    #[test]
    // Deduplicate removes consecutive repeated elements,
    // thus if the input is sorted dedup returns no duplicates
    fn test_deduplicate_load_series() {
        let mut v_all = vec![1., 2., 2., 2., 3., 4., 4., 5., 6., 6.];
        let v_unique = vec![1., 2., 3., 4., 5., 6.];
        let v_duplicates = vec![2., 2., 4., 6.];

        v_all.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let (dedup, duplicates) = v_all.partition_dedup_by(|a, b| a == b);

        let mut duplicates = duplicates.to_owned();
        duplicates.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // println!("test:\ndedup {:?} == {:?}?\nduplicates {:?} == {:?}?", dedup, v_unique, duplicates, v_duplicates);
        assert!(v_unique == dedup);
        assert!(v_duplicates == duplicates);
    }

    #[test]
    fn test_discharge_by_index() {
        let vall: Vec<f64> = (1..20).map(|n| n as f64).collect();
        let indices: Vec<usize> = vec![2, 3, 7, 12, 13, 18];
        let mut expected: Vec<f64> = (1..20).map(|n| n as f64).collect();
        for i in indices.iter().rev() {
            expected.remove(*i);
        }
        let vout = discharge_by_index(&vall, &indices);
        println!("vout\n{:?}\nexpected\n{:?}", vout, expected);
        assert!(compare_vecf64_approx(&vout, &expected));
    }

    #[test]
    fn test_setnan_by_index() {
        let mut vall: Vec<f64> = (1..20).map(|n| n as f64).collect();
        let indices: Vec<usize> = vec![2, 3, 7, 12, 13, 18];
        let mut expected: Vec<f64> = (1..20).map(|n| n as f64).collect();
        indices.iter().for_each(|i| expected[*i] = f64::NAN);
        setnan_by_index(&mut vall, &indices);
        assert!(compare_vecf64_approx(&vall, &expected));
    }

    #[test]
    // full processing test, including all the optional steps
    fn test_all_steps() {
        // define time zone for the test
        let mut timezone: i32 = -8;
        timezone *= 60 * 60;
        let timezone_fixed_offset = FixedOffset::east_opt(timezone).unwrap();

        // read the time series and adjust to the deifned time zone
        let mut tl = TimeLoad::from_csv(String::from("./test/timeload_raw.csv"));
        tl.time
            .iter_mut()
            .for_each(|t| *t = t.with_timezone(&timezone_fixed_offset));
        println!("{}", tl);

        // make sure the time series is ordered before processing
        tl.is_ordered();

        // make continuous by filling missing values with NANs, then assert continuity
        let mut ctl = tl.fill_missing_with_nan();
        ctl.is_ordered_and_continuous();
        println!("{}", ctl);

        // read bad datetimes and replace them with NANs
        let bad = read_bad_datetimes("./test/bad_datetimes.csv");
        ctl.replace_bad_datetimes_with_nan(bad);
        println!("{}", ctl);

        // define a daily interval over which values should be ignored (maintenance, etc.),
        // then these intervals to NAN
        let time_init = NaiveTime::parse_from_str("01:02", "%H:%M").unwrap();
        let time_stop = NaiveTime::parse_from_str("01:05", "%H:%M").unwrap();
        ctl.replace_bad_time_interval_with_nan(time_init, time_stop);
        println!("{}", ctl);

        // replace errors with NANs, in this case all the values above 99995.
        ctl.replace_errors_with_nan(99995.);
        println!("{}", ctl);

        // keep only load values within a specific range, set the outliers to NAN
        ctl.replace_outliers_with_nan(10000., 18000.);
        println!("{}", ctl);

        // find anomalies and set to NAN
        let (anomalies_indices, _) = find_anomalies(&ctl.load, 16usize, 8usize, 40.0f64);
        println!("indices of the anomalies:\n{:?}", anomalies_indices);
        let mut atl = TimeLoad::new(anomalies_indices.len());
        for i in anomalies_indices.iter() {
            atl.time.push(ctl.time.get(*i).unwrap().clone());
            atl.load.push(ctl.load.get(*i).unwrap().clone());
        }
        atl.to_csv("./test/timeload_anomalies.csv");

        setnan_by_index(&mut ctl.load[..], &anomalies_indices);

        // apply a weighted moving average to smooth the filtered time series
        let mavg_window = make_window(3., 1., 5usize);
        let smooth = mavg(&ctl.load[..], &mavg_window, 5 as usize, 80.);
        println!("{:?}", smooth);

        // in this case simply replace the original load series with the smooth one;
        // if preferred keep both and compare
        ctl.load = smooth;

        // plot the filtered and smooth load series
        ctl.plot_datetime("./test/timeload_processed.svg").unwrap();

        // save the filtered and smooth load series
        ctl.to_csv("./test/timeload_processed.csv");
    }

    #[test]
    // full processing test, including all the optional steps
    fn test_all_steps_parallel() {
        // define time zone for the test
        let mut timezone: i32 = -8;
        timezone *= 60 * 60;
        let timezone_fixed_offset = FixedOffset::east_opt(timezone).unwrap();

        // read the time series and adjust to the deifned time zone
        let mut tl = TimeLoad::from_csv(String::from("./test/parallel_timeload_raw.csv"));
        tl.time
            .iter_mut()
            .for_each(|t| *t = t.with_timezone(&timezone_fixed_offset));
        println!("{}", tl);

        // make sure the time series is ordered before processing
        tl.is_ordered();

        // make continuous by filling missing values with NANs, then assert continuity
        let mut ctl = tl.fill_missing_with_nan();
        ctl.is_ordered_and_continuous();
        println!("{}", ctl);

        // read bad datetimes and replace them with NANs
        let bad = read_bad_datetimes("./test/parallel_bad_datetimes.csv");
        ctl.replace_bad_datetimes_with_nan(bad);
        println!("{}", ctl);

        // define a daily interval over which values should be ignored (maintenance, etc.),
        // then these intervals to NAN
        let time_init = NaiveTime::parse_from_str("01:02", "%H:%M").unwrap();
        let time_stop = NaiveTime::parse_from_str("01:05", "%H:%M").unwrap();
        ctl.replace_bad_time_interval_with_nan(time_init, time_stop);
        println!("{}", ctl);

        // replace errors with NANs, in this case all the values above 99995.
        ctl.replace_errors_with_nan(99995.);
        println!("{}", ctl);

        // keep only load values within a specific range, set the outliers to NAN
        ctl.replace_outliers_with_nan(10000., 18000.);
        println!("{}", ctl);

        // find anomalies and set to NAN
        let (anomalies_indices, _) = find_anomalies(&ctl.load, 16usize, 8usize, 40.0f64);
        println!("indices of the anomalies:\n{:?}", anomalies_indices);
        let mut atl = TimeLoad::new(anomalies_indices.len());
        for i in anomalies_indices.iter() {
            atl.time.push(ctl.time.get(*i).unwrap().clone());
            atl.load.push(ctl.load.get(*i).unwrap().clone());
        }
        atl.to_csv("./test/parallel_timeload_anomalies.csv");

        setnan_by_index(&mut ctl.load[..], &anomalies_indices);

        // apply a weighted moving average to smooth the filtered time series
        let mavg_window = make_window(3., 1., 2usize);
        let smooth = mavg_parallel_fold(&ctl.load[..], &mavg_window);
        println!("{:?}", smooth);

        let correct_smooth = vec![
            f64::NAN,
            f64::NAN,
            13003.0,
            13004.0,
            13005.0,
            13006.0,
            13007.0,
            13008.0,
            13008.8,
            13009.6,
            13010.3,
            13011.1,
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

        assert! {compare_vecf64_approx(&smooth, &correct_smooth)};

        // in this case simply replace the original load series with the smooth one;
        // if preferred keep both and compare
        ctl.load = smooth;

        // plot the filtered and smooth load series
        ctl.plot_datetime("./test/parallel_timeload_processed.svg")
            .unwrap();

        // save the filtered and smooth load series
        ctl.to_csv("./test/parallel_timeload_processed.csv");
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
