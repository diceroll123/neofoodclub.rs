use crate::{
    chance::Chance,
    math::{binary_to_indices, build_chance_objects},
    nfc::NeoFoodClub,
};

#[derive(Debug, Clone)]
pub struct Odds {
    /// The Chance object with the highest odds value.
    pub best: Chance,

    /// The Chance object for busting. Can be None if this bet set is bustproof.
    pub bust: Option<Chance>,

    /// The Chance object with the highest probability.
    pub most_likely_winner: Chance,

    /// The sum of probabilities where you'd make a partial return.
    pub partial_rate: f64,

    /// A vector of Chance objects, sorted by probability from least to greatest.
    pub chances: Vec<Chance>,
}

impl Odds {
    pub fn new(nfc: &NeoFoodClub, array_indices: Vec<usize>) -> Self {
        let amount_of_bets = array_indices.len();
        let mut pirate_indices = Vec::<[u8; 5]>::with_capacity(amount_of_bets);
        let mut odds_values = Vec::<u32>::with_capacity(amount_of_bets);

        for index in array_indices.iter() {
            pirate_indices.push(binary_to_indices(nfc.round_dict_data().bins[*index]));
            odds_values.push(nfc.round_dict_data().odds[*index]);
        }

        let chances = build_chance_objects(&pirate_indices, &odds_values, nfc.probabilities());

        let best = chances
            .last()
            .expect("Chances vector should not be empty")
            .clone();

        let bust = chances.first().and_then(|bust_chance| {
            if bust_chance.value == 0 {
                Some(bust_chance.clone())
            } else {
                None
            }
        });

        let most_likely_winner = chances
            .iter()
            .filter(|o| o.value > 0)
            .max_by(|a, b| a.probability.total_cmp(&b.probability))
            .expect("Chances vector should not be empty")
            .clone();

        let partial_rate = chances
            .iter()
            .filter(|o| 0 < o.value && o.value < amount_of_bets as u32)
            .map(|o| o.probability)
            .sum();

        Self {
            best,
            bust,
            most_likely_winner,
            partial_rate,
            chances,
        }
    }
}
