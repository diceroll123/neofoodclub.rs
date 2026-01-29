use chrono::{DateTime, Duration, TimeDelta, TimeZone, Utc};
use chrono_tz::{OffsetComponents, Tz, US::Pacific};
use std::cmp::Ordering;
use std::sync::OnceLock;

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

/// Pre-allocated indices [0..3124] to avoid repeated allocation
static INDICES_3124: OnceLock<Box<[usize; 3124]>> = OnceLock::new();

fn get_indices_3124() -> &'static [usize; 3124] {
    INDICES_3124.get_or_init(|| {
        let mut arr = [0usize; 3124];
        for (i, v) in arr.iter_mut().enumerate() {
            *v = i;
        }
        Box::new(arr)
    })
}

/// Specialized argsort for 3124-element arrays (common in round_dict_data)
/// Uses a pre-allocated index array and generic comparator for better inlining
#[inline]
pub fn argsort_3124<T, F>(arr: &[T; 3124], compare: F) -> Box<[usize; 3124]>
where
    F: Fn(&T, &T) -> Ordering,
{
    let mut indices: Box<[usize; 3124]> = Box::new(*get_indices_3124());
    indices.sort_unstable_by(|&i, &j| compare(&arr[i], &arr[j]));
    indices
}

/// Specialized argsort for 3124-element slices
/// Panics if slice length != 3124
#[inline]
pub fn argsort_slice_3124<T, F>(arr: &[T], compare: F) -> Box<[usize; 3124]>
where
    F: Fn(&T, &T) -> Ordering,
{
    assert_eq!(arr.len(), 3124, "Slice must have exactly 3124 elements");
    let mut indices: Box<[usize; 3124]> = Box::new(*get_indices_3124());
    indices.sort_unstable_by(|&i, &j| compare(&arr[i], &arr[j]));
    indices
}

pub fn get_dst_offset(today: DateTime<Utc>) -> TimeDelta {
    let today_as_nst = Pacific.from_utc_datetime(&today.naive_utc());

    let yesterday = today_as_nst - Duration::try_days(1).unwrap();

    let today_offset = today_as_nst.offset().dst_offset();
    let yesterday_offset = yesterday.offset().dst_offset();

    match yesterday_offset.cmp(&today_offset) {
        Ordering::Less => TimeDelta::try_hours(1).unwrap(),
        Ordering::Greater => TimeDelta::try_hours(-1).unwrap(),
        Ordering::Equal => TimeDelta::zero(),
    }
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
