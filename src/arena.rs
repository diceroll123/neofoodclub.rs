use std::ops::{Add, Sub};

use crate::{
    food_adjustments::{NEGATIVE_FOOD, POSITIVE_FOOD},
    math::binary_to_indices,
    nfc::RoundData,
    pirates::Pirate,
};

pub const ARENA_NAMES: [&str; 5] = ["Shipwreck", "Lagoon", "Treasure", "Hidden", "Harpoon"];

#[derive(Debug, Clone)]
pub struct Arena {
    pub id: u8,
    pub pirates: Vec<Pirate>,
    pub odds: f64,
    pub winner: u8,
    pub foods: Option<[u8; 10]>,
}

impl Arena {
    pub fn new(id: u8, round_data: &RoundData) -> Arena {
        let mut odds = 0.;

        // the pirate index of the winning pirate
        let winner = round_data.winners.unwrap_or([0; 5])[id as usize];

        let mut pirates: Vec<Pirate> = Vec::with_capacity(5);
        for (index, pirate_id) in round_data.pirates[id as usize].iter().enumerate() {
            let p_id = pirate_id.to_owned();
            let current_odds = round_data.currentOdds[id as usize][index + 1];

            let mut pfa: Option<u8> = None;
            let mut nfa: Option<i8> = None;
            let mut fa: Option<i8> = None;
            if let Some(foods) = &round_data.foods {
                for food in &foods[id as usize] {
                    pfa = Some(
                        pfa.unwrap_or(0)
                            .add(POSITIVE_FOOD[p_id as usize - 1][*food as usize - 1]),
                    );
                    nfa = Some(
                        nfa.unwrap_or(0_i8)
                            .sub(NEGATIVE_FOOD[p_id as usize - 1][*food as usize - 1] as i8),
                    );
                }
                fa = Some(pfa.unwrap_or(0) as i8 + nfa.unwrap_or(0));
            }

            let is_winner = winner as usize == (index + 1);

            pirates.push(Pirate {
                id: p_id,
                arena_id: id,
                index: (index + 1) as u8,
                current_odds,
                opening_odds: round_data.openingOdds[id as usize][index + 1],
                fa,
                pfa,
                nfa,
                is_winner,
            });

            odds += 1. / current_odds as f64;
        }
        Arena {
            id,
            pirates,
            odds,
            winner,
            foods: round_data.foods.map(|f| f[id as usize]),
        }
    }

    /// Returns the name of the arena.
    pub fn get_name(&self) -> &'static str {
        ARENA_NAMES[self.id as usize]
    }

    /// Returns whether or not the arena is positive.
    pub fn is_positive(&self) -> bool {
        self.odds < 1.
    }

    /// Returns whether or not the arena is negative.
    pub fn is_negative(&self) -> bool {
        !self.is_positive()
    }

    /// Returns a vector of pirates in this arena sorted from least to greatest odds.
    pub fn best(&self) -> Vec<Pirate> {
        let mut pirates = self.pirates.clone();
        pirates.sort_by_key(|pirate| pirate.current_odds);
        pirates
    }

    /// Returns a vector of the IDs of the pirates in this arena.
    pub fn ids(&self) -> Vec<u8> {
        self.pirates.iter().map(|pirate| pirate.id).collect()
    }

    /// Returns the ratio of the arena.
    pub fn ratio(&self) -> f64 {
        1. / self.odds - 1.
    }

    /// Returns the pirate by index (0 through 3, inclusive).
    pub fn get_pirate_by_index(&self, index: u8) -> Option<&Pirate> {
        self.pirates.get(index as usize)
    }
}

#[derive(Debug, Clone)]
pub struct Arenas {
    pub arenas: Vec<Arena>,
}

impl Arenas {
    pub fn new(round_data: &RoundData) -> Arenas {
        let arenas: Vec<Arena> = (0..5).map(|i| Arena::new(i, round_data)).collect();
        Arenas { arenas }
    }

    /// Returns the desired pirate by ID.
    /// Will only be None if the ID is invalid.
    pub fn get_pirate_by_id(&self, id: u8) -> Option<&Pirate> {
        self.arenas
            .iter()
            .flat_map(|arena| arena.pirates.iter())
            .find(|pirate| pirate.id == id)
    }

    /// Returns a vector of pirates by ID.
    pub fn get_pirates_by_id(&self, ids: &[u8]) -> Vec<&Pirate> {
        ids.iter()
            .filter_map(|id| self.get_pirate_by_id(*id))
            .collect()
    }

    /// Returns all pirates in the arenas.
    pub fn get_all_pirates_flat(&self) -> Vec<&Pirate> {
        self.arenas
            .iter()
            .flat_map(|arena| arena.pirates.iter())
            .collect()
    }

    /// Returns a vector of pirates from a binary representation.
    pub fn get_pirates_from_binary(&self, binary: u32) -> Vec<&Pirate> {
        let indices = binary_to_indices(binary);
        self.arenas
            .iter()
            .filter_map(|arena| {
                let pirate_index = indices[arena.id as usize];
                if pirate_index == 0 {
                    None
                } else {
                    Some(&arena.pirates[(pirate_index - 1) as usize])
                }
            })
            .collect()
    }

    /// Returns a vector of all pirates in their arenas.
    pub fn get_all_pirates(&self) -> Vec<Vec<Pirate>> {
        self.arenas
            .iter()
            .map(|arena| arena.pirates.clone())
            .collect()
    }

    /// Returns the arenas sorted by best odds.
    pub fn best(&self) -> Vec<Arena> {
        let mut best: Vec<Arena> = self.arenas.clone();
        best.sort_by(|a, b| a.odds.total_cmp(&b.odds));
        best
    }

    /// Returns the IDs of the pirates in the arenas.
    pub fn pirate_ids(&self) -> Vec<Vec<u8>> {
        self.arenas.iter().map(|arena| arena.ids()).collect()
    }

    /// Returns the positive arenas, sorted by best odds.
    pub fn positives(&self) -> Vec<Arena> {
        let mut positives: Vec<Arena> = self
            .arenas
            .iter()
            .filter(|arena| arena.is_positive())
            .cloned()
            .collect();
        positives.sort_by(|a, b| a.odds.total_cmp(&b.odds));
        positives
    }

    /// Returns an arena by index. 0-4 inclusive.
    pub fn get_arena(&self, id: usize) -> Option<Arena> {
        // if id is invalid, return None
        self.arenas.get(id).cloned()
    }
}
