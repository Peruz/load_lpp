use chrono::prelude::*;
use load_lpp::make_window;
use load_lpp::mavg;
use load_lpp::process::parse_cli;
use load_lpp::read_bad_datetimes;
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
        min_load,
        max_load,
        bad_datetimes,
        bad_time_interval,
        timezone,
    ) = parse_cli();

    println!(
        "Reading time series in RFC3339 - ISO8601 and resetting to timezone {}",
        timezone
    );

    println!("> read data from {}", csvin.to_str().unwrap());
    let mut tw = TimeLoad::from_csv(csvin);

    let timezone_seconds = timezone * 60 * 60;
    let timezone_fixed_offset = FixedOffset::east(timezone_seconds);
    tw.time
        .iter_mut()
        .for_each(|t| *t = t.with_timezone(&timezone_fixed_offset));

    tw.is_ordered();

    println!("> fill missing values with nan");
    let mut ftw = tw.fill_missing_with_nan();

    println!("> check that the time series is continuous and ordered");
    ftw.is_ordered_and_continuous();

    if bad_datetimes.is_some() {
        let bdt = bad_datetimes.unwrap();
        let vec_bad_dateimes = read_bad_datetimes(&bdt);
        println!(
            "> found {} bad datetimes in {}, set them to nan",
            vec_bad_dateimes.len(),
            bdt.to_str().unwrap()
        );
        ftw.replace_bad_datetimes_with_nan(vec_bad_dateimes);
    }

    if bad_time_interval.is_some() {
        let t = bad_time_interval.unwrap();
        println!(
            "> consider daily times between {} and {} as invalid, set them to nan",
            t.0, t.1
        );
        ftw.replace_bad_time_interval_with_nan(t.0, t.1);
    }

    let largest_valid = 999994.;
    println!(
        "> consider all values larger than {} as error codes, set them to nan",
        largest_valid
    );
    ftw.replace_errors_with_nan(largest_valid);

    println!(
        "> consider outliers values below {} or above {}, set them to nan",
        min_load, max_load
    );
    ftw.replace_outliers_with_nan(min_load, max_load);

    println!("> apply moving average to smooth and fill nan");
    if side != 0 {
        let mavg_window = make_window(mavg_central_weight, mavg_side_weight, side);
        let smooth = mavg(
            &ftw.load[..],
            &mavg_window,
            mavg_max_missing_values,
            mavg_max_missing_pct_weight,
        );
        ftw.load = smooth;
    }

    println!("> save processed data to {}", csvout.to_str().unwrap());
    ftw.to_csv(csvout);
}
