use chrono::prelude::*;
use load_lpp::find_anomalies;
use load_lpp::load_process::parse_cli;
use load_lpp::make_window;
use load_lpp::mavg;
use load_lpp::read_bad_datetimes;
use load_lpp::setnan_by_index;
use load_lpp::TimeLoad;

fn main() {

    let (
        csvin,
        csvout,
        side,
        mavg_max_missing_values,
        mavg_max_missing_pct_weight,
        mavg_central_weight,
        mavg_side_weight,
        anomaly_detect,
        anomaly_width,
        anomaly_iqr,
        min_load,
        max_load,
        bad_datetimes,
        bad_time_interval,
        timezone,
        verbose,
    ) = parse_cli();

    println!(
        "Reading time series in RFC3339 - ISO8601 and resetting to timezone {}",
        timezone
    );

    if verbose {
        println!("csvin {:?}", csvin);
        println!("csvout {:?}", csvout);
        println!("side {}", side);
        println!("mavg_max_missing_values {}", mavg_max_missing_values);
        println!(
            "mavg_max_missing_pct_weight {}",
            mavg_max_missing_pct_weight
        );
        println!("mavg_central_weight {}", mavg_central_weight);
        println!("mavg_side_weight {}", mavg_side_weight);
        println!("anomaly_detect {}", anomaly_detect);
        println!("anomaly_width {}", anomaly_width);
        println!("anomaly_iqr {}", anomaly_iqr);
        println!("min_load {}", min_load);
        println!("max_load {}", max_load);
        println!("bad_datetimes {:?}", bad_datetimes);
        println!("bad_time_interval {:?}", bad_time_interval);
        println!("timezone {}", timezone);
        println!("verbose {}", verbose);
    }

    println!("> read data from {}", csvin.to_str().unwrap());
    let mut tl = TimeLoad::from_csv(csvin);

    let timezone_seconds = timezone * 60 * 60;
    let timezone_fixed_offset = FixedOffset::east_opt(timezone_seconds).unwrap();
    tl.time
        .iter_mut()
        .for_each(|t| *t = t.with_timezone(&timezone_fixed_offset));

    tl.is_ordered();

    println!("> fill missing values with nan");
    let mut ftl = tl.fill_missing_with_nan();

    println!("> check that the time series is continuous and ordered");
    ftl.is_ordered_and_continuous();

    if bad_datetimes.is_some() {
        let bdt = bad_datetimes.unwrap();
        let vec_bad_dateimes = read_bad_datetimes(&bdt);
        println!(
            "> found {} bad datetimes in {}, set them to nan",
            vec_bad_dateimes.len(),
            bdt.to_str().unwrap()
        );
        ftl.replace_bad_datetimes_with_nan(vec_bad_dateimes);
    }

    if bad_time_interval.is_some() {
        let t = bad_time_interval.unwrap();
        println!(
            "> consider daily times between {} and {} as invalid, set them to nan",
            t.0, t.1
        );
        ftl.replace_bad_time_interval_with_nan(t.0, t.1);
    }

    let largest_valid = 999994.;
    println!(
        "> consider all values larger than {} as error codes, set them to nan",
        largest_valid
    );
    ftl.replace_errors_with_nan(largest_valid);

    println!(
        "> consider outliers values below {} or above {}, set them to nan",
        min_load, max_load
    );
    ftl.replace_outliers_with_nan(min_load, max_load);

    // Optional anomaly detection, save them to file so that they can be added to the bad datetimes.
    // Meanwhile, set values to nan.
    // Require at least half of the window width to be valid load values, otherwise skip it.
    println!("> anomomaly detection is {}", anomaly_detect);
    if anomaly_detect {
        let min_data_anomaly = anomaly_width / 2usize;
        let (anomalies_indices, _) =
            find_anomalies(&ftl.load, anomaly_width, min_data_anomaly, anomaly_iqr);
        let mut atl = TimeLoad::new(anomalies_indices.len());
        for i in anomalies_indices.iter() {
            atl.time.push(ftl.time.get(*i).unwrap().clone());
            atl.load.push(ftl.load.get(*i).unwrap().clone());
        }
        atl.to_csv("./timeload_anomalies.csv");
        setnan_by_index(&mut ftl.load[..], &anomalies_indices);
    }

    println!("> apply moving average to smooth and fill nan");
    if side != 0 {
        let mavg_window = make_window(mavg_central_weight, mavg_side_weight, side);
        let smooth = mavg(
            &ftl.load[..],
            &mavg_window,
            mavg_max_missing_values,
            mavg_max_missing_pct_weight,
        );
        ftl.load = smooth;
    }

    println!("> save processed data to {}", csvout.to_str().unwrap());
    ftl.to_csv(csvout);
}
