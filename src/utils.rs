use chrono::prelude::*;
use rayon::prelude::*;
use std::cmp::PartialOrd;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::{error::Error, fmt};
use nalgebra::{DVector, DMatrix};
use plotly::{Plot, Scatter};


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
                // println!(
                //     "setting to NAN; {} missing data with limit {}, {} missing window weight with limit {}",
                //     missing_v, max_missing_v, missing_w, max_missing_w,
                // );
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


pub fn awat_regression_plot(data: DVector<f64>, model:DVector<f64>) {
    let l = data.column(0).len();
    let base: Vec<i32> = (0..l).map(|e| e as i32).collect();
    let mut plot = Plot::new();
    let trace_data = Scatter::new(base.clone(), data.data.as_vec().to_vec()).name("data");
    plot.add_trace(trace_data);
    let trace_model = Scatter::new(base.clone(), model.data.as_vec().to_vec()).name("fit");
    plot.add_trace(trace_model);
    plot.show();
}


// Adaptive Window and Adaptive Threshold.
// Peters et al., 2014. Hydrol. Earth Syst. Sci., 18, 1189–1198, 2014
// This assumes either ET or P is active, not both at the same time.
// d is the threshold, smaller changes are ignored.
// w is the width of the moving-average window.
// The variable moving average can address the variability of
// the noise intensity (wind, temp, etc.) and signal strength (rain and strong ET).
pub fn awat_regression(v: &[f64], len_w: usize) -> (u8, f64) {
    let k_max = 6;  // this is standard
    let plot = false;

    let len_v: usize = v.len();
    assert!(
        len_w < len_v,
        "length of moving average window > length of vector"
    );
    assert!(
        len_w % 2 == 1,
        "the moving average window has an even number of elements; \
        it should be odd to have a central element"
    );

    let base_lower: i32 = - (len_w as i32 - 1) / 2;
    let base_upper: i32 = (len_w as i32 + 1) / 2;
    let base: Vec<i32> = (base_lower..base_upper).collect();

    // init and populate g matrix
    let mut g_all = DMatrix::from_element(len_w, k_max + 1, 1.);
    for i in 1..7 {
        let dv  = DVector::from_iterator(len_w, base.clone().iter().map(|e| e.pow(i) as f64));
        g_all.set_column(i as usize, &dv);
    }
    // alternative initialization
    // let g_all = DMatrix::from_columns(&[dv0, dv1, dv2, dv3, dv4, dv5, dv6]);

    let d = DVector::from_column_slice(&v[0..len_w]);
    let ssd = d.dot(&d);

    let mut solutions = DMatrix::zeros(k_max + 1, k_max);
    let mut models: DMatrix<f64> = DMatrix::zeros(len_w, k_max);
    let mut ssrs: Vec<f64> = Vec::with_capacity(6);
    let mut aics: Vec<f64> = Vec::with_capacity(6);

    for k in 1..(k_max + 1) {

        let g = g_all.columns(0, k + 1);

        let s: DVector<f64> = (g.transpose() * &g).try_inverse().unwrap() * g.transpose() * d.clone();

        for (i, &e) in s.iter().enumerate() {
            solutions[(i, k - 1)] = e
        }

        let sol: DVector<f64> = solutions.column(k - 1).into();

        let model = &g_all * sol;
        models.set_column(k - 1, &model);

        let res = &model - &d;
        let ssr = res.dot(&res);
        ssrs.push(ssr);

        // Akaike's information criterion
        let n = (k + 1) as f64;
        let r = len_w as f64;
        let aic = r * f64::ln(ssr / r) + (2. * n) + ((2. * n * (n + 1.)) / (r - n - 1.));
        aics.push(aic);

    }

    // find best acis, and associated solution and order
    let mut min_index = 0;
    let mut min_value = aics[0];
    for (i, &e) in aics.iter().enumerate() {
        if e < min_value {
            min_value = e;
            min_index = i;
        }
    }
    // let sol_best: DVector<f64> = solutions.column(min_index).into();
    let mod_best: DVector<f64> = models.column(min_index).into();
    let ssr_best = ssrs[min_index];
    let k_best = (min_index + 1) as u8;

    if plot {
        awat_regression_plot(d, mod_best)
    }

    let b = ssr_best / ssd;

    (k_best, b)
}


// A configurable and automatic detection of anomalous periods
// based on the interquartile range (IQR).
// Anomalies can be periods that have to be removed or the actual events of interest.
// For example, the reported anomalies can be appended to the bad datetimes input
// and thus be removed in the successive processing iteration.
//
// Run a rolling window of width `window_width` over the vector `v`.
// Make sure that:
// 1] `window_width` is larger than the minimum number of data required by the user `min_window_data`
// 2] `min_window_data` is statistically sufficient for calculating the IQR, see `MIN_DATA_IQR`.
//
// Return unique values of the indices and loads that fell in an anomalous window.
pub fn find_anomalies(
    v: &[f64],
    window_width: usize,
    min_window_data: usize,
    max_iqr: f64,
) -> (Vec<usize>, Vec<f64>) {
    // Initial length checks for consistent lengths
    pub const MIN_DATA_IQR: usize = 6usize;
    if min_window_data < MIN_DATA_IQR {
        panic!(
            "find_anomalies: more than {} data are required for the IQR calculation",
            MIN_DATA_IQR
        );
    }
    if min_window_data > window_width {
        panic!("find_anomalies: impossible to proceed as window_width < min_window_data");
    }
    let mut anomalies_index: Vec<usize> = Vec::new();
    let indices: Vec<usize> = (0..v.len()).collect();
    for (wl, wi) in v.windows(window_width).zip(indices.windows(window_width)) {
        let (_ql, _qu, iqr) = match calculate_iqr(wl, min_window_data) {
            Ok(res) => res,
            Err(_e) => {
                continue;
            }
        };
        if iqr > max_iqr {
            anomalies_index.append(&mut wi.to_owned());
        }
    }
    // Anomalous windows may give duplicates, keep only unique indices:
    // first, order the indices so that multiple duplicates will be consecutive,
    // then deduplicate more quickly and in-place.
    anomalies_index.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let (anomalies_index_dedup, _) = anomalies_index.partition_dedup_by(|a, b| a == b);
    let anomalies_index_dedup = anomalies_index_dedup.to_vec();
    let mut anomalies_load: Vec<f64> = Vec::new();
    for i in anomalies_index_dedup.iter() {
        anomalies_load.push(v[*i]);
    }
    return (anomalies_index_dedup, anomalies_load);
}

// Calculate the lower and upper quartiles
// using the linear method (R-7) to calculate the IQR.
// Note, no + 1 here because of the zero-starting indexing, i.e.,
// h = (N - 1) * q + 1  => (N - 1) * q
// This is analogous to the default method chosen by NumPy.
//
// Use map to dereference the f64 and usize,
// as they are cheap and implement copy.
// This gives a Vec that owns its elements.
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

pub fn mean_or_nan(v: &Vec<f64>) -> f64 {
    
    let mut contains_nan = false;
    v.iter().for_each(|f| {
          if f.is_nan() {
              contains_nan = true
          }
    });

    let mean = if contains_nan {
        f64::NAN
    } else if v.len() == 0usize {
        f64::NAN
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    };

    mean
}


pub fn compare_f64_exact(a: f64, b: f64) -> bool {
    (a.is_nan() && b.is_nan()) || (a == b)
}

pub fn compare_vecf64_exact(va: &[f64], vb: &[f64]) -> bool {
    (va.len() == vb.len()) && va.iter().zip(vb).all(|(a, b)| compare_f64_exact(*a, *b))
}

pub fn compare_f64_approx(a: f64, b: f64, max: f64) -> bool {
    (a.is_nan() && b.is_nan()) || ((a - b).abs() < max)
}

pub fn compare_vecf64_approx(va: &[f64], vb: &[f64]) -> bool {
    (va.len() == vb.len())
        && va
            .iter()
            .zip(vb)
            .all(|(a, b)| compare_f64_approx(*a, *b, 0.1f64))
}

/// Return a new vector without the elements associated with the given indices.
/// The indices are expected to be sorted or partially sorted:
/// sort them completely and then take advantage of that.
/// This avoids random indexing-access the vector,
/// which can reduce cache misses and the number of comparisons.
/// This is used for removing bad datetimes and anomalies.
pub fn discharge_by_index<T: Copy>(ve: &[T], vi: &[usize]) -> Vec<T> {
    let mut vout: Vec<T> = Vec::with_capacity(ve.len());
    let mut vi = vi.to_vec();
    vi.sort();
    assert!(vi[vi.len() - 1usize] < ve.len());
    let mut vi_iter = vi.iter();
    let mut ve_iter = ve.iter().enumerate();
    while let Some(i) = vi_iter.next() {
        loop {
            let (vei, vee) = ve_iter.next().unwrap();
            if vei < *i {
                vout.push(*vee);
            } else if vei == *i {
                break;
            } else {
                panic!("indices error: {} > {}", vei, i);
            }
        }
    }
    while let Some((_, vee)) = ve_iter.next() {
        vout.push(*vee);
    }
    return vout;
}

/// Set to NAN the elements of the given vector at the given indices.
/// The indices are expected to be sorted or partially sorted:
/// sort them completely and then take advantage of that.
/// This avoids random indexing-access the vector,
/// which can reduce cache misses and the number of comparisons.
/// This is used for removing bad datetimes and anomalies.
pub fn setnan_by_index(ve: &mut [f64], vi: &[usize]) {
    if vi.len() == 0 {
        return;
    }
    let mut vi = vi.to_vec();
    vi.sort();
    assert!(vi[vi.len() - 1usize] < ve.len());
    let mut vi_iter = vi.iter();
    let mut ve_iter = ve.iter_mut().enumerate();
    while let Some(i) = vi_iter.next() {
        loop {
            let (vei, vee) = ve_iter.next().unwrap();
            if vei < *i {
                continue;
            } else if vei == *i {
                *vee = f64::NAN;
                break;
            } else {
                panic!("indices error: {} > {}", vei, i);
            }
        }
    }
}

// An Error type for empty TimeLoad
#[derive(Debug)]
pub struct EmptyTimeLoad();
impl fmt::Display for EmptyTimeLoad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Found an empty TimeLoad")
    }
}
impl Error for EmptyTimeLoad {}

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
