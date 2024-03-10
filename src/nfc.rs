use crate::arena::Arenas;
use crate::bets::Bets;
use crate::math::{
    make_round_dicts, pirates_binary, RoundDictData, BET_AMOUNT_MAX, BET_AMOUNT_MIN, BIT_MASKS,
};
use crate::modifier::{Modifier, ModifierFlags};
use crate::utils::argsort_by;
use itertools::Itertools;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Deserialize)]
struct RoundDataRaw {
    // as an intermediate step, we use this struct to deserialize the JSON
    foods: String,
    round: u16,
    start: Option<String>,
    pirates: String,
    openingOdds: String,
    currentOdds: String,
    winners: String,
    timestamp: Option<String>,
    lastChange: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoundData {
    pub(super) foods: Option<[[u8; 10]; 5]>,
    pub(super) round: u16,
    pub(super) start: Option<String>,
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

        let mut nfc = NeoFoodClub {
            round_data: round_data.clone(),
            arenas,
            bet_amount: None,
            stds,
            data,
            modifier: modifier.unwrap_or_default(),
        };

        nfc.set_bet_amount(bet_amount);

        nfc
    }

    pub fn set_bet_amount(&mut self, amount: Option<u32>) {
        self.bet_amount = amount.map(|x| x.clamp(BET_AMOUNT_MIN, BET_AMOUNT_MAX));
    }

    pub fn from_json(
        json: &str,
        bet_amount: Option<u32>,
        model: Option<ProbabilityModel>,
        modifier: Option<Modifier>,
    ) -> NeoFoodClub {
        let round_data: RoundData = serde_json::from_str(json).expect("Invalid JSON.");

        NeoFoodClub::new(round_data, bet_amount, model, modifier)
    }

    pub fn from_url(
        url: &str,
        bet_amount: Option<u32>,
        model: Option<ProbabilityModel>,
        modifier: Option<Modifier>,
    ) -> NeoFoodClub {
        let parts = url.split('#').collect::<Vec<&str>>();

        if parts.len() != 2 {
            panic!("No relevant NeoFoodClub-like URL data found.");
        }

        let use_modifier = modifier.unwrap_or_default();
        let cc_perk = parts[0].ends_with("/15/") || use_modifier.is_charity_corner();
        let new_modifier = Modifier::new(
            use_modifier.value
                | if cc_perk {
                    ModifierFlags::CHARITY_CORNER.bits()
                } else {
                    0
                },
            None,
        );

        let temp: RoundDataRaw = serde_qs::from_str(parts[1]).expect("Invalid query string.");

        let round_data = RoundData {
            foods: serde_json::from_str(&temp.foods).expect("Invalid foods JSON."),
            round: temp.round,
            start: temp.start,
            pirates: serde_json::from_str(&temp.pirates).expect("Invalid pirates JSON."),
            openingOdds: serde_json::from_str(&temp.openingOdds)
                .expect("Invalid openingOdds JSON."),
            currentOdds: serde_json::from_str(&temp.currentOdds)
                .expect("Invalid currentOdds JSON."),
            winners: serde_json::from_str(&temp.winners).expect("Invalid winners JSON."),
            timestamp: temp.timestamp,
            changes: None,
            lastChange: temp.lastChange,
        };

        NeoFoodClub::new(round_data, bet_amount, model, Some(new_modifier))
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

    pub fn start(&self) -> Option<String> {
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

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.round_data).expect("Failed to serialize to JSON.")
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
                let maxbets = &self.data.maxbets.iter().map(|&x| x.min(bet_amount) as f64);
                let new_ers: Vec<f64> = maxbets
                    .clone()
                    .zip(ers.iter())
                    .map(|(maxbet, er)| maxbet * er - maxbet)
                    .collect();

                ers = new_ers;
            }
        }

        // by default, this orders from least to greatest
        let mut binding = argsort_by(&ers, &|a: &f64, b: &f64| a.total_cmp(b));

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

        let mut binding = argsort_by(odds, &|a: &u32, b: &u32| a.cmp(b));

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
        let values: Vec<u16> = (0..3124).collect();

        let chosen_values: Vec<u16> = values
            .choose_multiple(&mut rng, self.max_amount_of_bets())
            .cloned()
            .collect();

        let mut bets = Bets::new(self, chosen_values, None);
        bets.fill_bet_amounts(self);
        bets
    }

    /// Creates a Bets object that consists of max-TER bets.
    pub fn make_max_ter_bets(&self) -> Bets {
        let indices = self.max_ter_indices(self.max_amount_of_bets());

        let mut bets = Bets::new(self, indices, None);
        bets.fill_bet_amounts(self);
        bets
    }

    /// Creates a Bets object that consists of a gambit of the given 5-bet pirates binary.
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
        bets.fill_bet_amounts(self);
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

        match winners_binary {
            0 => None,
            _ => Some(self.make_gambit_bets(winners_binary)),
        }
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
        bets.fill_bet_amounts(self);
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
                let best_arena = &positives[0];

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
                let (best_arena, second_best_arena) = (&positives[0], &positives[1]);

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

                let (best_arena, second_best_arena, third_best_arena) =
                    (&positives[0], &positives[1], &positives[2]);

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
                let odds = bets.odds_values(self);
                let lowest = odds.iter().min().expect("Odds vector is empty, somehow");

                let bet_amounts: Vec<Option<u32>> =
                    odds.iter().map(|odd| Some(amount * lowest / odd)).collect();

                bets.bet_amounts = Some(bet_amounts);
            }

            return Some(bets);
        }

        None
    }

    pub fn make_tenbet_bets(&self, pirates_binary: u32) -> Result<Bets, String> {
        let mut amount_of_pirates = 0;
        for mask in BIT_MASKS.iter() {
            let arena_pirates = (pirates_binary & mask).count_ones();

            if arena_pirates > 1 {
                return Err("You can only pick 1 pirate per arena.".to_string());
            }

            amount_of_pirates += arena_pirates;
        }

        if amount_of_pirates == 0 {
            return Err("You must pick at least 1 pirate, and at most 3.".to_string());
        }

        if amount_of_pirates > 3 {
            return Err("You must pick 3 pirates at most.".to_string());
        }

        let max_ter_indices = self.max_ter_indices(3124);

        let mut bins = Vec::with_capacity(self.max_amount_of_bets());

        for index in max_ter_indices.iter() {
            let bin = self.data.bins[*index as usize];
            if bin & pirates_binary == pirates_binary {
                bins.push(bin);
                if bins.len() == bins.capacity() {
                    break;
                }
            }
        }

        Ok(Bets::from_binaries(self, bins))
    }

    /// Creates a Bets object translated from a bets hash.
    pub fn make_bets_from_hash(&self, hash: &str) -> Bets {
        let mut bets = Bets::from_hash(self, hash);

        bets.fill_bet_amounts(self);

        bets
    }

    /// Creates a Bets object translated from a bets binary vector.
    pub fn make_bets_from_binaries(&self, binaries: Vec<u32>) -> Bets {
        let mut bets = Bets::from_binaries(self, binaries);

        bets.fill_bet_amounts(self);

        bets
    }

    /// Creates a Bets object translated from a bets indices vector.
    pub fn make_bets_from_indices(&self, indices: Vec<[u8; 5]>) -> Bets {
        let mut bets = Bets::from_indices(self, indices);

        bets.fill_bet_amounts(self);

        bets
    }
}

impl NeoFoodClub {
    // win-related stuff

    pub fn get_win_units(&self, bets: &Bets) -> u32 {
        let winners_binary = self.winners_binary();

        if winners_binary == 0 {
            return 0;
        }

        let mut units = 0;
        for i in bets.array_indices.iter() {
            let bet_bin = self.data.bins[*i as usize];
            if bet_bin & winners_binary == bet_bin {
                units += self.data.odds[*i as usize];
            }
        }

        units
    }

    pub fn get_win_np(&self, bets: &Bets) -> u32 {
        let Some(bet_amounts) = bets.bet_amounts.as_ref() else {
            return 0;
        };

        let winners_binary = self.winners_binary();

        if winners_binary == 0 {
            return 0;
        }

        let mut np = 0;

        for (bet_index, array_index) in bets.array_indices.iter().enumerate() {
            let bet_bin = self.data.bins[*array_index as usize];
            if bet_bin & winners_binary == bet_bin {
                np += (self.data.odds[*array_index as usize] * bet_amounts[bet_index].unwrap_or(0))
                    .clamp(0, 1_000_000);
            }
        }

        np
    }
}

impl NeoFoodClub {
    // URL-related stuff
    pub fn make_url(&self, bets: &Bets) -> String {
        let use_15 = self.modifier.is_charity_corner() || bets.len() > 10;

        let mut url = format!(
            "https://neofood.club/{}#round={}&b={}",
            if use_15 { "15/" } else { "" },
            self.round(),
            bets.bets_hash()
        );

        if let Some(amounts_hash) = bets.amounts_hash() {
            url.push_str(&format!("&a={}", amounts_hash));
        }

        url
    }

    pub fn copy(&self) -> NeoFoodClub {
        let round_data = self.round_data.clone();
        let bet_amount = self.bet_amount;
        let arenas = self.arenas.clone();
        let stds = self.stds;
        let data = self.data.clone();
        let modifier = self.modifier.clone();

        NeoFoodClub {
            round_data,
            bet_amount,
            arenas,
            stds,
            data,
            modifier,
        }
    }
}
