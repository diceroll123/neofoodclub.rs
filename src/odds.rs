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
    pub fn new(nfc: &NeoFoodClub, array_indices: Vec<u16>) -> Self {
        let mut pirate_indices = Vec::<[u8; 5]>::with_capacity(array_indices.len());
        let mut odds_values = Vec::<u32>::with_capacity(array_indices.len());

        for index in array_indices.iter() {
            pirate_indices.push(binary_to_indices(nfc.data.bins[*index as usize]));
            odds_values.push(nfc.data.odds[*index as usize]);
        }

        let chances = build_chance_objects(&pirate_indices, &odds_values, nfc.stds);

        let best = chances
            .last()
            .expect("Chances vector is empty, somehow")
            .clone();

        let bust = chances.first().and_then(|bust_chance| {
            if bust_chance.value == 0 {
                Some(bust_chance.clone())
            } else {
                None
            }
        });

        let amount_of_bets = array_indices.len() as u32;
        let most_likely_winner = chances
            .iter()
            .max_by(|a, b| a.probability.total_cmp(&b.probability))
            .expect("Chances vector is empty, somehow")
            .clone();

        let partial_rate = chances
            .iter()
            .filter(|o| 0 < o.value && o.value < amount_of_bets)
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
