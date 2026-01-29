use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use chrono_tz::US::Pacific;
use std::cmp::Ordering;

/// ```
/// let arr = vec![5, 4, 3, 2, 1, 6, 7, 8, 9, 0];
/// let indices = neofoodclub::utils::argsort_by(&arr, &|a: &u8, b: &u8| a.cmp(b));
/// assert_eq!(indices, vec![9, 4, 3, 2, 1, 0, 5, 6, 7, 8]);
/// ```
pub fn argsort_by<T>(arr: &[T], compare: &dyn Fn(&T, &T) -> Ordering) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..arr.len()).collect();
    indices.sort_unstable_by(move |&i, &j| compare(&arr[i], &arr[j]));
    indices
}

#[inline]
pub fn timestamp_to_utc(timestamp: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(timestamp)
        .unwrap()
        .with_timezone(&Utc)
}

#[inline]
pub fn convert_from_utc_to_nst(utc: DateTime<Utc>) -> DateTime<Tz> {
    Pacific.from_utc_datetime(&utc.naive_utc())
}
