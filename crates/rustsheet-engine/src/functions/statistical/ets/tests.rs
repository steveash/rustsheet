use super::{
    build_series, completion_setting, detect_season, interpolate, Aggregate, Point, SeriesOptions,
    MAX_ACF_WORK,
};
use crate::expressions::token::Error;

const DEFAULT: SeriesOptions = SeriesOptions {
    interpolate: true,
    aggregate: Aggregate::Average,
};

fn numbers(values: &[f64]) -> Vec<Point> {
    values.iter().copied().map(Point::Number).collect()
}

fn steps(count: usize) -> Vec<Point> {
    (1..=count).map(|t| Point::Number(t as f64)).collect()
}

#[test]
fn interpolation_fills_gaps_and_ends() {
    let slots = [None, Some(4.0), None, None, Some(13.0), None];
    assert_eq!(interpolate(&slots), vec![4.0, 4.0, 7.0, 10.0, 13.0, 13.0]);
    assert_eq!(interpolate(&[None, None]), vec![0.0, 0.0]);
}

#[test]
fn a_period_four_signal_has_season_four() {
    let y: Vec<f64> = (0..24)
        .map(|t| [1.0, 5.0, 2.0, 8.0][t % 4] + 0.1 * t as f64)
        .collect();
    assert_eq!(detect_season(&y), Ok(4));
}

#[test]
fn a_straight_line_has_no_season() {
    let y: Vec<f64> = (0..24).map(|t| 3.0 + 2.0 * t as f64).collect();
    assert_eq!(detect_season(&y), Ok(0));
    assert_eq!(detect_season(&[1.0, 2.0, 3.0]), Ok(0));
}

#[test]
fn season_detection_is_capped() {
    // n lags n/2 below MAX_SEASON, so the cap is n * n/2 > MAX_ACF_WORK: n = 16,385.
    let n = 16_385;
    assert!(n * (n / 2) > MAX_ACF_WORK && (n - 1) * ((n - 1) / 2) <= MAX_ACF_WORK);
    let y: Vec<f64> = (0..n).map(|t| (t % 7) as f64).collect();
    assert_eq!(detect_season(&y), Err(Error::NUM));
}

#[test]
fn series_pairs_sorts_and_combines_duplicates() {
    let values = numbers(&[10.0, 2.0, 4.0, 6.0]);
    let timeline = numbers(&[3.0, 1.0, 1.0, 2.0]);
    let series = build_series(&values, &timeline, DEFAULT).map(|s| s.y);
    assert_eq!(series, Ok(vec![3.0, 6.0, 10.0]));
    let summed = SeriesOptions {
        aggregate: Aggregate::Sum,
        ..DEFAULT
    };
    assert_eq!(
        build_series(&values, &timeline, summed).map(|s| s.y),
        Ok(vec![6.0, 6.0, 10.0])
    );
}

#[test]
fn series_completes_missing_points() {
    let values = numbers(&[1.0, 2.0, 4.0, 5.0]);
    let timeline = numbers(&[1.0, 2.0, 4.0, 5.0]);
    assert_eq!(
        build_series(&values, &timeline, DEFAULT).map(|s| s.y),
        Ok(vec![1.0, 2.0, 3.0, 4.0, 5.0])
    );
    let zeros = SeriesOptions {
        interpolate: false,
        ..DEFAULT
    };
    assert_eq!(
        build_series(&values, &timeline, zeros).map(|s| s.y),
        Ok(vec![1.0, 2.0, 0.0, 4.0, 5.0])
    );
}

#[test]
fn series_errors() {
    let four = steps(4);
    assert_eq!(
        build_series(&numbers(&[1.0; 3]), &four, DEFAULT).map(|_| ()),
        Err(Error::NA)
    );
    let text = [
        Point::Number(1.0),
        Point::Other,
        Point::Number(3.0),
        Point::Number(4.0),
    ];
    assert_eq!(
        build_series(&text, &four, DEFAULT).map(|_| ()),
        Err(Error::VALUE)
    );
    // One time stamp, an uneven step, and more than 30% missing are #NUM!.
    assert_eq!(
        build_series(&numbers(&[1.0, 2.0]), &numbers(&[5.0, 5.0]), DEFAULT).map(|_| ()),
        Err(Error::NUM)
    );
    assert_eq!(
        build_series(&numbers(&[1.0; 3]), &numbers(&[1.0, 2.0, 3.5]), DEFAULT).map(|_| ()),
        Err(Error::NUM)
    );
    assert_eq!(
        build_series(&numbers(&[1.0; 3]), &numbers(&[1.0, 2.0, 6.0]), DEFAULT).map(|_| ()),
        Err(Error::NUM)
    );
}

#[test]
fn a_blank_row_is_skipped_and_a_blank_value_is_missing() {
    let values = [
        Point::Number(1.0),
        Point::Blank,
        Point::Blank,
        Point::Number(4.0),
        Point::Number(5.0),
    ];
    let timeline = [
        Point::Number(1.0),
        Point::Blank,
        Point::Number(2.0),
        Point::Number(3.0),
        Point::Number(4.0),
    ];
    assert_eq!(
        build_series(&values, &timeline, DEFAULT).map(|s| s.y),
        Ok(vec![1.0, 2.5, 4.0, 5.0])
    );
}

#[test]
fn monthly_dates_are_one_step_apart() {
    // 2024-01-31, 2024-02-29, 2024-03-31, 2024-04-30: month ends.
    let timeline = numbers(&[45322.0, 45351.0, 45382.0, 45412.0]);
    let values = numbers(&[1.0, 2.0, 3.0, 4.0]);
    assert_eq!(
        build_series(&values, &timeline, DEFAULT).map(|s| s.y),
        Ok(vec![1.0, 2.0, 3.0, 4.0])
    );
}

#[test]
fn settings() {
    assert_eq!(completion_setting(1.9), Some(true));
    assert_eq!(completion_setting(0.0), Some(false));
    assert_eq!(completion_setting(2.0), None);
    assert_eq!(Aggregate::from_setting(0.5), Some(Aggregate::Average));
    assert_eq!(Aggregate::from_setting(7.0), Some(Aggregate::Sum));
    assert_eq!(Aggregate::from_setting(8.0), None);
    assert_eq!(Aggregate::from_setting(-1.0), None);
}
