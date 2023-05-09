use std::cmp::Ordering;

use ndarray::{ArrayBase, Data, Ix1};
/// ```
/// use ndarray::{array};
/// let arr = array![5, 4, 3, 2, 1, 6, 7, 8, 9, 0];
/// let indices = neofoodclub::utils::argsort_by(&arr, |a, b| a.cmp(b));
/// assert_eq!(indices, vec![9, 4, 3, 2, 1, 0, 5, 6, 7, 8]);
/// ```
pub fn argsort_by<S, F>(arr: &ArrayBase<S, Ix1>, mut compare: F) -> Vec<usize>
where
    S: Data,
    F: FnMut(&S::Elem, &S::Elem) -> Ordering,
{
    let mut indices: Vec<usize> = (0..arr.len()).collect();
    indices.sort_unstable_by(move |&i, &j| compare(&arr[i], &arr[j]));
    indices
}
