use chrono::prelude::*;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::{error::Error, fmt};

/// If longer than one week, keep year, month and day, drop hours;
/// if not, but longer than one day, add hours.
/// Otherwise, shorter than one day, keep also minutes.
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

/// Read a list of bad datetimes to skip, always from RFC 3339 - ISO 8601 format.
pub fn read_bad_datetimes<P>(fin: P) -> Vec<DateTime<FixedOffset>>
where
    P: AsRef<Path>,
{
    let file = File::open(fin).unwrap();
    let buf = BufReader::new(file);
    let mut bad_datetimes: Vec<DateTime<FixedOffset>> = Vec::new();
    for l in buf.lines() {
        let l_unwrap = match l {
            Ok(l_ok) => l_ok,
            Err(l_err) => {
                println!("Err, could not read/unwrap line {}", l_err);
                continue;
            }
        };
        bad_datetimes.push(DateTime::parse_from_rfc3339(&l_unwrap).unwrap());
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

pub fn make_window(w_central: f64, w_side: f64, side: usize) -> Vec<f64> {
    let w_step = (w_central - w_side) / (side as f64);
    let up = (0..side + 1).map(|n| w_side + (n as f64 * w_step));
    let down = up.clone().rev().skip(1);
    let updown = up.chain(down).collect();
    updown
}

// Flexible Weighted Moving Average implementation with parameters to handle the maximum missing information.
/// Roll the weighted moving window w over the data v,
/// also filling the NAN values with the weighted average when possible:
/// 1) sufficient number of data, i.e., number missing data under the window < max_missing_v;
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

// Weighted Moving Average implementation for long windows and
// with limited number of expected missing values in the time series.
// This is a parallel implementation of the moving average
// that splits the multiplication step from the successive sum.
// This allows SIMD parallelism, but requires second loop over the window for the sum.
// The SIMD optimization, in addition to the multi-threading, has been confirmed by the assembly.
pub fn mavg_parallel_simd(v: &[f64], w: &[f64]) -> Vec<f64> {
    let len_v: usize = v.len();
    let len_w: usize = w.len();
    assert!(
        len_w < len_v,
        "length of moving average window > length of vector"
    );
    assert!(
        len_w % 2 == 1,
        "the moving average window has an even number of elements; \
        it should be odd to have a central element"
    );
    let sum_all_w: f64 = w.iter().sum();
    let side: usize = (len_w - 1) / 2;
    let mut vout: Vec<f64> = vec![f64::NAN; len_v];
    v.par_windows(len_w as usize)
        .zip(vout[side as usize..].par_iter_mut())
        .for_each(|(window, vout_e)| {
            let product: Vec<f64> = window
                .iter()
                .zip(w)
                .map(|(win_e, wt_e)| win_e * wt_e)
                .collect();
            let sum: f64 = product.iter().sum();
            *vout_e = sum / sum_all_w;
        });
    vout
}

// Weighted Moving Average implementation for long windows,
// for limited number of expected missing values and edge devices with limited memory.
// This is a parallel implementation of the moving average that
// allows the sum of the weighted loads to be directly executed,
// i.e., pair-wise multiplication proceed together with the sum.
pub fn mavg_parallel_fold(v: &[f64], w: &[f64]) -> Vec<f64> {
    let len_v: usize = v.len();
    let len_w: usize = w.len();
    assert!(
        len_w < len_v,
        "length of moving average window > length of vector"
    );
    assert!(
        len_w % 2 == 1,
        "the moving average window has an even number of elements; \
        it should be odd to have a central element"
    );
    let sum_all_w: f64 = w.iter().sum();
    let side: usize = (len_w - 1) / 2;
    let mut vout: Vec<f64> = vec![f64::NAN; len_v];
    v.par_windows(len_w as usize)
        .zip(vout[side as usize..].par_iter_mut())
        .for_each(|(window, vout_e)| {
            *vout_e = window
                .iter()
                .zip(w)
                .map(|(win_e, wt_e)| win_e * wt_e)
                .fold(0., |acc, x| acc + x)
                / sum_all_w;
        });
    vout
}

// A configurable and automatic detection of anomal events
// which can be used for reporting or filtering.
// The reported anomalies can be appended to the bad datetimes input
// and thus be removed in the successive processing iteration.
//
// use map to dereference the f64 and usize,
// as they are cheap and implement copy.
// This gives a Vec that owns its elements.
// All the elements are finite and sorted,
// ready for the IQR range calculation.
//
pub fn find_anomalis(
    v: &[f64],
    window_width: usize,
    min_window_data: usize,
    max_iqr: f64,
) -> (Vec<usize>, Vec<f64>) {
    let min_window_data_accepted = 6usize;
    if window_width < min_window_data {
        panic!("find_anomalies: impossible to proceed as window_width > min_window_data");
    }
    if min_window_data < min_window_data_accepted {
        panic!(
            "find_anomalies: more than {} data are required",
            min_window_data_accepted
        );
    }
    let indices: Vec<usize> = (0..v.len()).collect();
    let mut anomalies_index: Vec<usize> = Vec::new();
    let mut anomalies_load: Vec<f64> = Vec::new();
    for (wl, wi) in v.windows(window_width).zip(indices.windows(window_width)) {
        println!("new window: {:?} {:?}", wl, wi);
        let (ql, qu, iqr) = match calculate_iqr(wl, min_window_data_accepted) {
            Ok(res) => res,
            Err(e) => {
                println!("{}", e);
                continue
            },
        };
        if iqr > max_iqr {
            anomalies_index.append(&mut wi.to_owned());
            anomalies_load.append(&mut wl.to_owned());
            println!("iqr {}, upper {} and lower {}\nfound anomaly in window:\n{:?}\n{:?}", ql, qu, iqr, wi, wl);
        }

    }
    return(anomalies_index, anomalies_load)

}

// Calculate the lower and upper quartiles
// using the linear method (R-7) to calculate the IQR.
// Note, no + 1 here because of the zero-starting indexing, i.e.,
// h = (N - 1) * q + 1  => (N - 1) * q
// This is analogous to the default method chosen by NumPy.
pub fn calculate_iqr(s: &[f64], min_len: usize) -> Result<(f64, f64, f64), LenErr> {
    let mut v: Vec<f64> = s.iter().filter(|n| n.is_finite()).map(|n| *n).collect();
    let v_len = v.len();

    if v_len < min_len {
        let err = LenErr {
            min_len: Some(min_len),
            got_len: v_len,
            max_len: None,
        };
        return Err(err);
    }
    v.sort_by(|a, b| a.partial_cmp(&b).unwrap());
    let hl = (v_len as f64 - 1.) * 0.25;
    let hu = (v_len as f64 - 1.) * 0.75;
    let hl_int = hl.floor() as usize;
    let hl_fract = hl.fract();
    let hu_int = hu.floor() as usize;
    let hu_fract = hu.fract();
    let ql_int = v[hl_int];
    let qu_int = v[hu_int];
    let ql_fract = (v[hl_int + 1usize] - v[hl_int]) * hl_fract;
    let qu_fract = (v[hu_int + 1usize] - v[hu_int]) * hu_fract;
    let ql = ql_int + ql_fract;
    let qu = qu_int + qu_fract;
    let iqr = qu - ql;
    return Ok((ql, qu, iqr));
}

// An Error type for handling length requirements,
// often needed in time series and statistics.
#[derive(Debug)]
pub struct LenErr {
    pub min_len: Option<usize>,
    pub got_len: usize,
    pub max_len: Option<usize>,
}
impl Error for LenErr {}
impl fmt::Display for LenErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Invalid length, got {}, required is >= {:?} and <= {:?}",
            self.got_len, self.min_len, self.max_len
        )
    }
}
