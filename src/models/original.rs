use crate::nfc::RoundData;

#[derive(Debug, Clone)]
pub struct OriginalModel {}

impl OriginalModel {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(round_data: &RoundData) -> [[f64; 5]; 5] {
        make_probabilities(round_data.openingOdds)
    }
}

pub fn make_probabilities(odds: [[u8; 5]; 5]) -> [[f64; 5]; 5] {
    let mut std = [[1.0, 0.0, 0.0, 0.0, 0.0]; 5];
    let mut min = [[1.0, 0.0, 0.0, 0.0, 0.0]; 5];
    let mut max = [[1.0, 0.0, 0.0, 0.0, 0.0]; 5];
    // let mut used = [[1.0, 0.0, 0.0, 0.0, 0.0]; 5];

    // turns out we only use _std values in the python implementation of NFC
    // keeping the _used math to avoid confusion between NFC impls
    // however, if we use this Rust code on the frontend of neofood.club
    // that's the best time to expose this.

    for arena in 0..5 {
        let mut min_prob: f64 = 0.0;
        let mut max_prob: f64 = 0.0;

        for pirate in 1..5 {
            let pirate_odd = odds[arena][pirate];
            if pirate_odd == 13 {
                min[arena][pirate] = 0.0;
                max[arena][pirate] = 1.0 / 13.0;
            } else if pirate_odd == 2 {
                min[arena][pirate] = 1.0 / 3.0;
                max[arena][pirate] = 1.0;
            } else {
                let p_o: f64 = pirate_odd as f64;
                min[arena][pirate] = 1.0 / (1.0 + p_o);
                max[arena][pirate] = 1.0 / p_o;
            }

            min_prob += min[arena][pirate];
            max_prob += max[arena][pirate];
        }

        for pirate in 1..5 {
            let min_original: f64 = min[arena][pirate];
            let max_original: f64 = max[arena][pirate];

            min[arena][pirate] = f64::max(min_original, 1.0 + max_original - max_prob);
            max[arena][pirate] = f64::min(max_original, 1.0 + min_original - min_prob);
            std[arena][pirate] = match odds[arena][pirate] {
                13 => 0.05,
                _ => (min[arena][pirate] + max[arena][pirate]) / 2.0,
            };
        }

        for rectify_level in 2..13 {
            let mut rectify_count: f64 = 0.0;
            let mut std_total: f64 = 0.0;
            let mut rectify_value: f64 = 0.0;
            let mut max_rectify_value: f64 = 1.0;

            for pirate in 1..5 {
                std_total += std[arena][pirate];
                if odds[arena][pirate] <= rectify_level {
                    rectify_count += 1.0;
                    rectify_value += std[arena][pirate] - min[arena][pirate];
                    max_rectify_value =
                        f64::min(max_rectify_value, max[arena][pirate] - min[arena][pirate]);
                }
            }

            if std_total == 1.0 {
                break;
            }

            if !(std_total - rectify_value > 1.0
                || rectify_count == 0.0
                || max_rectify_value * rectify_count < rectify_value + 1.0 - std_total)
            {
                rectify_value += 1.0 - std_total;
                rectify_value /= rectify_count;
                for pirate in 1..5 {
                    if odds[arena][pirate] <= rectify_level {
                        std[arena][pirate] = min[arena][pirate] + rectify_value;
                    }
                }
                break;
            }
        }

        // let mut return_sum = 0.0;
        // for pirate in 1..5 {
        //     used[arena][pirate] = std[arena][pirate];
        //     return_sum += used[arena][pirate];
        // }

        // for pirate in 1..5 {
        //     used[arena][pirate] /= return_sum;
        // }
    }

    std
}
