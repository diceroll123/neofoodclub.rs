use chrono::{DateTime, Duration, TimeDelta, TimeZone, Utc};
use chrono_tz::{OffsetComponents, Tz, US::Pacific};
use std::cmp::Ordering;
use std::sync::OnceLock;

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
