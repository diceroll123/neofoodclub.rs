use crate::{
    chance::Chance,
    math::{binary_to_indices, build_chance_objects},
    nfc::NeoFoodClub,
};

#[derive(Debug, Clone)]
pub struct Odds {
    /// A vector of Chance objects, sorted by probability from least to greatest.
    chances: Vec<Chance>,

    /// Amount of bets in the set.
    amount_of_bets: u32,
}

impl Odds {
    pub fn new(nfc: &NeoFoodClub, array_indices: &[usize]) -> Self {
        let data = nfc.round_dict_data();
        let (pirate_indices, odds_values): (Vec<[u8; 5]>, Vec<u32>) = array_indices
            .iter()
            .map(|&index| (binary_to_indices(data.bins[index]), data.odds[index]))
            .unzip();

        let chances = build_chance_objects(&pirate_indices, &odds_values, nfc.probabilities());

        Self {
            chances,
            amount_of_bets: array_indices.len() as u32,
        }
    }

    /// The Chance object with the highest probability.
    pub fn most_likely_winner(&self) -> Chance {
        self.chances
            .iter()
            .filter(|c| c.value > 0)
            .max_by(|a, b| a.probability.total_cmp(&b.probability))
            .expect("Chances vector should not be empty")
            .clone()
    }

    /// The Chance object with the highest odds value.
    pub fn best(&self) -> Chance {
        self.chances
            .last()
            .expect("Chances vector should not be empty")
            .clone()
    }

    /// The Chance object for busting. Can be None if this bet set is bustproof.
    pub fn bust(&self) -> Option<Chance> {
        self.chances.first().filter(|c| c.value == 0).cloned()
    }

    /// The sum of probabilities where you'd make a partial return.
    pub fn partial_rate(&self) -> f64 {
        self.chances
            .iter()
            .filter(|c| (1..self.amount_of_bets).contains(&c.value))
            .map(|c| c.probability)
            .sum()
    }

    /// Returns a reference to the vector of Chance objects.
    pub fn chances(&self) -> &[Chance] {
        &self.chances
    }
}
