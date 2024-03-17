use chrono::{DateTime, Duration, TimeDelta, TimeZone, Utc};
use chrono_tz::{OffsetComponents, US::Pacific};
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

pub fn get_dst_offset(today: DateTime<Utc>) -> TimeDelta {
    let today_as_nst = Pacific.from_utc_datetime(&today.naive_utc());

    let yesterday = today_as_nst - Duration::try_days(1).unwrap();

    let today_offset = today_as_nst.offset().dst_offset();
    let yesterday_offset = yesterday.offset().dst_offset();

    if yesterday_offset < today_offset {
        return TimeDelta::try_hours(1).unwrap();
    } else if yesterday_offset > today_offset {
        return TimeDelta::try_hours(-1).unwrap();
    }

    TimeDelta::zero()
}
