use std::ops::Mul;

use crate::arena::Arenas;
use crate::bets::Bets;
use crate::math::{
    make_round_dicts, pirates_binary, RoundDictData, BET_AMOUNT_MAX, BET_AMOUNT_MIN,
};
use crate::modifier::Modifier;
use crate::utils::argsort_by;
use itertools::Itertools;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::Result;

use crate::models::multinomial_logit::MultinomialLogitModel;
use crate::models::original::OriginalModel;
use crate::pirates::Pirate;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Change {
    pub(super) t: String,
    pub(super) new: u8,
    pub(super) old: u8,
    pub(super) arena: u8,
    pub(super) pirate: u8,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoundData {
    pub(super) foods: Option<[[u8; 10]; 5]>,
    pub(super) round: u16,
    pub(super) start: String,
    pub(super) pirates: [[u8; 4]; 5],
    pub(super) currentOdds: [[u8; 5]; 5],
    pub(super) openingOdds: [[u8; 5]; 5],
    pub(super) winners: Option<[u8; 5]>,
    pub(super) timestamp: Option<String>,
    pub(super) changes: Option<Vec<Change>>,
    pub(super) lastChange: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub enum ProbabilityModel {
    #[default]
    OriginalModel,
    MultinomialLogitModel,
}

#[derive(Debug, Clone)]
pub struct NeoFoodClub {
    pub round_data: RoundData,
    pub bet_amount: Option<u32>,
    pub arenas: Arenas,
    pub stds: [[f64; 5]; 5],
    pub data: RoundDictData,
    pub modifier: Modifier,
}

impl NeoFoodClub {
    // constructor stuff
    pub fn new(
        round_data: RoundData,
        bet_amount: Option<u32>,
        model: Option<ProbabilityModel>,
        modifier: Option<Modifier>,
    ) -> NeoFoodClub {
        let arenas = Arenas::new(&round_data);

        let stds = match model.unwrap_or_default() {
            ProbabilityModel::OriginalModel => OriginalModel::new(&round_data),
            ProbabilityModel::MultinomialLogitModel => MultinomialLogitModel::new(&arenas),
        };

        let data = make_round_dicts(stds, round_data.currentOdds);

        NeoFoodClub {
            round_data: round_data.clone(),
            arenas,
            bet_amount: bet_amount.map(|x| x.clamp(BET_AMOUNT_MIN, BET_AMOUNT_MAX)),
            stds,
            data,
            modifier: modifier.unwrap_or_default(),
        }
    }

    pub fn from_json(
        json: &str,
        bet_amount: Option<u32>,
        model: Option<ProbabilityModel>,
        modifier: Option<Modifier>,
    ) -> Result<NeoFoodClub> {
        let round_data: Result<RoundData> = serde_json::from_str(json);
        match round_data {
            Ok(round_data) => Ok(NeoFoodClub::new(round_data, bet_amount, model, modifier)),
            Err(e) => {
                println!("Error: {}", e);
                Err(e)
            }
        }
    }
}

impl NeoFoodClub {
    // winner-related stuff

    /// Returns the indices of the winning pirates, if any.
    /// If there are no winners, returns a [0; 5] vector.
    pub fn winners(&self) -> [u8; 5] {
        match &self.round_data.winners {
            Some(winners) => *winners,
            None => [0; 5],
        }
    }

    /// Returns the binary representation of the winning pirates.
    /// Zero means no pirates won yet.
    pub fn winners_binary(&self) -> u32 {
        pirates_binary(self.winners())
    }

    /// Returns a vector of the winning pirates, if any.
    pub fn winning_pirates(&self) -> Option<Vec<&Pirate>> {
        let bin = self.winners_binary();

        if bin == 0 {
            return None;
        }

        Some(self.arenas.get_pirates_from_binary(bin))
    }

    /// Returns whether or not the round is over.
    /// A round is over if there are winners.
    pub fn is_over(&self) -> bool {
        if self.round_data.winners.is_none() {
            return false;
        }
        self.winners()[0] != 0
    }
}

impl NeoFoodClub {
    // getters from round_data
    pub fn round(&self) -> u16 {
        self.round_data.round
    }

    pub fn start(&self) -> String {
        self.round_data.start.clone()
    }

    pub fn current_odds(&self) -> [[u8; 5]; 5] {
        self.round_data.currentOdds
    }

    pub fn opening_odds(&self) -> [[u8; 5]; 5] {
        self.round_data.openingOdds
    }

    pub fn timestamp(&self) -> Option<String> {
        self.round_data.timestamp.clone()
    }

    pub fn changes(&self) -> Option<Vec<Change>> {
        self.round_data.changes.clone()
    }

    pub fn last_change(&self) -> Option<String> {
        self.round_data.lastChange.clone()
    }

    pub fn foods(&self) -> Option<[[u8; 10]; 5]> {
        self.round_data.foods
    }
}

impl NeoFoodClub {
    // Bets indices related stuff

    /// Returns the maximum amount of bets you can place.
    /// Normally, this would be 10.
    /// If the modifier has the charity corner perk, this is 15.
    pub fn max_amount_of_bets(&self) -> usize {
        if self.modifier.is_charity_corner() {
            15
        } else {
            10
        }
    }

    /// Returns max-TER indices.
    /// `amount` is the number of indices to return. (normally, this would be 10)
    fn max_ter_indices(&self, amount: usize) -> Vec<u16> {
        let mut ers = self.data.ers.clone();

        let general = self.modifier.is_general();
        let reverse = self.modifier.is_reverse();

        if !general {
            if let Some(bet_amount) = self.bet_amount {
                // if there's a bet amount, we use Net Expected instead of Expected Return
                let maxbets = &self.data.maxbets.map(|&x| x.min(bet_amount) as f64);
                let new_ers = maxbets.mul(&ers) - maxbets;
                ers = new_ers;
            }
        }

        // by default, this orders from least to greatest
        let mut binding = argsort_by(&ers, |a, b| a.total_cmp(b));

        // since it's reversed to begin with, we reverse it if
        // the modifier does not have the reverse flag
        if !reverse {
            binding.reverse();
        }

        binding
            .iter()
            .take(amount)
            .cloned()
            .map(|i| i as u16)
            .collect()
    }

    /// Returns sorted indices of odds
    /// If `descending` is true, returns highest to lowest.
    /// If `descending` is false, returns lowest to highest.
    fn get_sorted_odds_indices(&self, descending: bool, amount: usize) -> Vec<u16> {
        let odds = &self.data.odds;

        let mut binding = argsort_by(odds, |a, b| a.cmp(b));

        if descending {
            binding.reverse();
        }

        binding
            .iter()
            .take(amount)
            .cloned()
            .map(|i| i as u16)
            .collect()
    }

    /// Return the binary representation of the highest expected return full-arena bet.
    fn get_highest_er_full_bet(&self) -> u32 {
        let max_ter_indices = self.max_ter_indices(3124);

        for index in max_ter_indices.iter() {
            let bin = self.data.bins[*index as usize];
            if bin.count_ones() == 5 {
                return bin;
            }
        }

        // this should never happen
        panic!("No full-arena bet found.");
    }

    /// Returns all full-arena indices.
    fn all_full_arenas(&self) -> Vec<u16> {
        // we know there are 1024 full-arena bets
        let mut full_arenas: Vec<u16> = Vec::with_capacity(1024);

        for (index, bin) in self.data.bins.iter().enumerate() {
            if bin.count_ones() == 5 {
                full_arenas.push(index as u16);
            }
        }

        full_arenas
    }
}

impl NeoFoodClub {
    // bets-related stuff

    /// Creates a Bets object that consists of all bets.
    /// This is mostly for debugging purposes.
    pub fn make_all_bets(&self) -> Bets {
        Bets::new(self, (0..3124).collect_vec(), None)
    }

    /// Creates a Bets object that consists of random bets.
    /// Following these bets is not recommended.
    pub fn make_random_bets(&self) -> Bets {
        let mut rng = rand::thread_rng();
        let mut values: Vec<u16> = (0..3124).collect();

        values.shuffle(&mut rng);

        values.truncate(self.max_amount_of_bets());

        let mut bets = Bets::new(self, values, None);
        bets.fill_bet_amounts();
        bets
    }

    /// Creates a Bets object that consists of max-TER bets.
    pub fn make_maxter_bets(&self) -> Bets {
        let indices = self.max_ter_indices(self.max_amount_of_bets());

        let mut bets = Bets::new(self, indices, None);
        bets.fill_bet_amounts();
        bets
    }

    pub fn make_gambit_bets(&self, pirates_binary: u32) -> Bets {
        if pirates_binary.count_ones() != 5 {
            panic!("Pirates binary must have 5 pirates.");
        }

        let all_indices = self.get_sorted_odds_indices(true, 3124);

        let mut indices = Vec::<u16>::new();

        let max_amount_of_bets = self.max_amount_of_bets();

        // get indices of all bets that contain the pirates in the pirates_binary
        for index in all_indices.iter() {
            let bin = self.data.bins[*index as usize];
            if bin & pirates_binary == bin {
                indices.push(*index);
            }

            if indices.len() == max_amount_of_bets {
                break;
            }
        }

        let mut bets = Bets::new(self, indices, None);
        bets.fill_bet_amounts();
        bets
    }

    /// Creates a Bets object that consists of the best gambit bets.
    /// Basically just gambit bets with the highest expected return.
    pub fn make_best_gambit_bets(&self) -> Bets {
        let max_ter_pirates = self.get_highest_er_full_bet();

        self.make_gambit_bets(max_ter_pirates)
    }

    /// Creates a Bets object that consists of winning gambit bets.
    /// Pretty much the best bets you can make for a given round.
    pub fn make_winning_gambit_bets(&self) -> Option<Bets> {
        let winners_binary = self.winners_binary();

        if winners_binary == 0 {
            return None;
        }

        Some(self.make_gambit_bets(winners_binary))
    }

    /// Picks a random full-arena bet and makes a gambit out of it
    pub fn make_random_gambit_bets(&self) -> Bets {
        let mut rng = rand::thread_rng();
        let index = *self
            .all_full_arenas()
            .choose(&mut rng)
            .expect("No full-arena bets found, somehow");
        let bin = self.data.bins[index as usize];

        self.make_gambit_bets(bin)
    }

    /// Creates a Bets object that consits of "crazy" bets.
    /// Crazy bets consist of randomly-selected, full-arena bets.
    /// Following these bets is not recommended.
    pub fn make_crazy_bets(&self) -> Bets {
        let mut rng = rand::thread_rng();
        let mut crazy_bet_indices: Vec<u16> = self.all_full_arenas();

        crazy_bet_indices.shuffle(&mut rng);

        crazy_bet_indices.truncate(self.max_amount_of_bets());

        let mut bets = Bets::new(self, crazy_bet_indices, None);
        bets.fill_bet_amounts();
        bets
    }

    /// Creates a Bets object that consists of bustproof bets.
    /// Returns None if there are no positive arenas.
    pub fn make_bustproof_bets(&self) -> Option<Bets> {
        let positives = self.arenas.positives();

        if positives.is_empty() {
            return None;
        }

        let bets = match positives.len() {
            1 => {
                // If only one arena is positive, we place 1 bet on each of the pirates of that arena. Total bets = 4.
                let best_arenas = self.arenas.best();
                let best_arena = best_arenas[0];

                let binaries: Vec<u32> = best_arena
                    .pirates
                    .iter()
                    .map(|pirate| pirate.binary())
                    .collect();

                Some(Bets::from_binaries(self, binaries))
            }
            2 => {
                // If two arenas are positive, we place 1 bet on each of the three worst pirates of the best arena and
                // 1 bet on each of the pirates of the second arena + the best pirate of the best arena. Total bets = 7
                let best_arenas = self.arenas.best();
                let (best_arena, second_best_arena) = (&best_arenas[0], &best_arenas[1]);

                let best_pirate_binary = best_arena.best()[0].binary();

                let binaries: Vec<u32> = best_arena.best()[1..]
                    .iter()
                    .map(|pirate| pirate.binary())
                    .chain(
                        second_best_arena
                            .pirates
                            .iter()
                            .map(|pirate| pirate.binary() | best_pirate_binary),
                    )
                    .collect();

                Some(Bets::from_binaries(self, binaries))
            }
            3..=5 => {
                //  If three arenas are positive, we place 1 bet on each of the three worst pirates of the best arena,
                //  If four or more arenas are positive, we only play the three best arenas, seen below
                //  1 bet on each of the three worst pirates of the second arena + the best pirate of the best arena,
                //  and 1 bet on each of the pirates of the third arena + the best pirate of the best arena + the best pirate
                //  of the second arena. Total bets = 10.

                let best_arenas = self.arenas.best();
                let (best_arena, second_best_arena, third_best_arena) =
                    (&best_arenas[0], &best_arenas[1], &best_arenas[2]);

                let best_pirate_binary = best_arena.best()[0].binary();
                let second_best_pirate_binary = second_best_arena.best()[0].binary();

                let binaries: Vec<u32> = best_arena.best()[1..]
                    .iter()
                    .map(|pirate| pirate.binary())
                    .chain(
                        second_best_arena.best()[1..]
                            .iter()
                            .map(|pirate| pirate.binary() | best_pirate_binary),
                    )
                    .chain(third_best_arena.pirates.iter().map(|pirate| {
                        pirate.binary() | best_pirate_binary | second_best_pirate_binary
                    }))
                    .collect();

                Some(Bets::from_binaries(self, binaries))
            }
            _ => None,
        };

        // give it bet amounts
        if let Some(mut bets) = bets {
            if let Some(amount) = self.bet_amount {
                let odds = bets.odds_values();
                let lowest = odds.iter().min().expect("Odds vector is empty, somehow");

                let bet_amounts: Vec<u32> = odds.iter().map(|odd| amount * lowest / odd).collect();

                bets.amounts = Some(bet_amounts);
            }

            return Some(bets);
        }

        None
    }
}

impl NeoFoodClub {
    // win-related stuff

    pub fn get_win_units(&self, bets: &Bets) -> u32 {
        let mut units = 0;
        let winners_binary = self.winners_binary();

        for i in bets.array_indices.iter() {
            let bet_bin = self.data.bins[*i as usize];
            if bet_bin & winners_binary == bet_bin {
                units += self.data.odds[*i as usize];
            }
        }

        units
    }

    pub fn get_win_np(&self, bets: &Bets) -> u32 {
        let Some(bet_amounts) = bets.amounts.as_ref() else {
            return 0;
        };

        let winners_binary = self.winners_binary();

        let mut np = 0;

        for (bet_index, array_index) in bets.array_indices.iter().enumerate() {
            let bet_bin = self.data.bins[*array_index as usize];
            if bet_bin & winners_binary == bet_bin {
                np += (self.data.odds[*array_index as usize] * bet_amounts[bet_index])
                    .clamp(0, 1_000_000);
            }
        }

        np
    }
}

impl NeoFoodClub {
    // URL-related stuff
    pub fn make_url(&self, bets: &Bets) -> String {
        let mut url = "https://neofood.club".to_string();

        url.push_str("/#round=");
        url.push_str(&self.round().to_string());

        url.push_str("&b=");
        url.push_str(&bets.bets_hash());

        if let Some(amounts_hash) = bets.amounts_hash() {
            url.push_str("&a=");
            url.push_str(&amounts_hash);
        }

        url
    }
}
