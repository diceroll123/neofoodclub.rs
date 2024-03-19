use std::collections::HashMap;

use crate::arena::Arenas;
use crate::bets::Bets;
use crate::math::{
    make_round_dicts, pirates_binary, RoundDictData, BET_AMOUNT_MAX, BET_AMOUNT_MIN, BIT_MASKS,
};
use crate::modifier::{Modifier, ModifierFlags};
use crate::oddschange::OddsChange;
use crate::round_data::RoundData;
use crate::utils::{argsort_by, convert_from_utc_to_nst, get_dst_offset, timestamp_to_utc};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use itertools::Itertools;
use querystring::stringify;
use rand::seq::SliceRandom;
use serde::Deserialize;

use crate::models::multinomial_logit::MultinomialLogitModel;
use crate::models::original::OriginalModel;
use crate::pirates::Pirate;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct RoundDataRaw {
    // as an intermediate step, we use this struct to deserialize the JSON
    foods: Option<String>,
    round: u16,
    start: Option<String>,
    pirates: String,
    openingOdds: String,
    currentOdds: String,
    winners: Option<String>,
    timestamp: Option<String>,
    lastChange: Option<String>,
}

/// The probability model to use when calculating bets.
#[derive(Debug, Clone, Default)]
pub enum ProbabilityModel {
    #[default]
    OriginalModel,
    MultinomialLogitModel,
}

/// A struct to represent the NeoFoodClub object.
/// This object contains all the data needed to calculate bets,
/// and methods to create URLs.
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
        mut round_data: RoundData,
        bet_amount: Option<u32>,
        model: Option<ProbabilityModel>,
        modifier: Option<Modifier>,
    ) -> NeoFoodClub {
        validate_round_data(&round_data);

        let use_modifier = modifier.unwrap_or_default();

        if use_modifier.modified() {
            // if the modifier has custom odds or opening odds or custom time, we apply it to the round data
            round_data = use_modifier.apply(&round_data);
        }

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
            modifier: use_modifier,
        };

        nfc.set_bet_amount(bet_amount);

        nfc
    }

    /// Sets the bet amount
    pub fn set_bet_amount(&mut self, amount: Option<u32>) {
        self.bet_amount = amount.map(|x| x.clamp(BET_AMOUNT_MIN, BET_AMOUNT_MAX));
    }

    /// Creates a NeoFoodClub object from a JSON string.
    /// This is generally the entrypoint for creating a NeoFoodClub object.
    pub fn from_json(
        json: &str,
        bet_amount: Option<u32>,
        model: Option<ProbabilityModel>,
        modifier: Option<Modifier>,
    ) -> NeoFoodClub {
        let round_data: RoundData = serde_json::from_str(json).expect("Invalid JSON.");

        NeoFoodClub::new(round_data, bet_amount, model, modifier)
    }

    /// Creates a NeoFoodClub object from a NeoFoodClub-like URL.
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
            None,
        );

        let temp: RoundDataRaw = serde_qs::from_str(parts[1]).expect("Invalid query string.");

        let round_data = RoundData {
            foods: temp
                .foods
                .map(|x| serde_json::from_str(&x).expect("Invalid foods JSON.")),
            round: temp.round,
            start: temp.start,
            pirates: serde_json::from_str(&temp.pirates).expect("Invalid pirates JSON."),
            openingOdds: serde_json::from_str(&temp.openingOdds)
                .expect("Invalid openingOdds JSON."),
            currentOdds: serde_json::from_str(&temp.currentOdds)
                .expect("Invalid currentOdds JSON."),
            winners: temp
                .winners
                .map(|x| serde_json::from_str(&x).expect("Invalid winners JSON.")),
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

    /// Returns the round number.
    pub fn round(&self) -> u16 {
        self.round_data.round
    }

    /// Returns the start time of the round in ISO 8601 format as a string.
    /// If the start time is not available, returns None.
    pub fn start(&self) -> Option<String> {
        self.round_data.start.clone()
    }

    /// Returns the start time of the round in NST.
    /// If the start time is not available, returns None.
    pub fn start_nst(&self) -> Option<DateTime<Tz>> {
        self.start()
            .map(|start| convert_from_utc_to_nst(timestamp_to_utc(&start)))
    }

    /// Returns the start time of the round in UTC.
    /// If the start time is not available, returns None.
    pub fn start_utc(&self) -> Option<DateTime<Utc>> {
        self.start().map(|start| timestamp_to_utc(&start))
    }

    /// Returns the current odds.
    pub fn current_odds(&self) -> [[u8; 5]; 5] {
        self.round_data.currentOdds
    }

    /// Returns the opening odds.
    pub fn opening_odds(&self) -> [[u8; 5]; 5] {
        self.round_data.openingOdds
    }

    /// Returns the timestamp of the round in ISO 8601 format as a string.
    pub fn timestamp(&self) -> Option<String> {
        self.round_data.timestamp.clone()
    }

    /// Returns the timestamp of the round in NST.
    /// If the timestamp is not available, returns None.
    pub fn timestamp_nst(&self) -> Option<DateTime<Tz>> {
        self.round_data.timestamp_nst()
    }

    /// Returns the timestamp of the round in UTC.
    /// If the timestamp is not available, returns None.
    pub fn timestamp_utc(&self) -> Option<DateTime<Utc>> {
        self.round_data.timestamp_utc()
    }

    /// Returns the pirate IDs, as a 2D array.
    /// The first dimension is the arena index, and the second dimension is the pirate index.
    pub fn pirates(&self) -> [[u8; 4]; 5] {
        self.round_data.pirates
    }

    /// Returns the changes of the round.
    pub fn changes(&self) -> Option<Vec<OddsChange>> {
        self.round_data.changes.clone()
    }

    /// Returns the last change of the round in ISO 8601 format as a string.
    /// If the last change is not available, returns None.
    pub fn last_change(&self) -> Option<String> {
        self.round_data.lastChange.clone()
    }

    /// Returns the last change of the round in NST.
    /// If the last change is not available, returns None.
    pub fn last_change_nst(&self) -> Option<DateTime<Tz>> {
        self.last_change()
            .map(|last_change| convert_from_utc_to_nst(timestamp_to_utc(&last_change)))
    }

    /// Returns the last change of the round in UTC.
    /// If the last change is not available, returns None.
    pub fn last_change_utc(&self) -> Option<DateTime<Utc>> {
        self.last_change()
            .map(|last_change| timestamp_to_utc(&last_change))
    }

    /// Returns the foods of the round.
    /// If the foods are not available, returns None.
    pub fn foods(&self) -> Option<[[u8; 10]; 5]> {
        self.round_data.foods
    }

    /// Returns the custom odds in the modifier, if any.
    pub fn custom_odds(&self) -> Option<&HashMap<u8, u8>> {
        self.modifier.custom_odds.as_ref()
    }

    /// Returns whether or not the modifier has made changes to the round data.
    pub fn modified(&self) -> bool {
        self.modifier.modified()
    }

    /// Returns whether or not the round is outdated.
    pub fn is_outdated_lock(&self) -> bool {
        let Some(start_date) = self.start_utc() else {
            return true;
        };

        let day_after = start_date
            .checked_add_signed(chrono::Duration::try_days(1).unwrap())
            .unwrap();

        let difference = get_dst_offset(day_after);

        let now = chrono::Utc::now();

        !(start_date <= now && now <= day_after + difference)
    }

    /// Serialize the round data to JSON.
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

    /// Returns sorted indices of probabilities
    /// If `descending` is true, returns highest to lowest.
    /// If `descending` is false, returns lowest to highest.
    fn get_sorted_probs_indices(&self, descending: bool, amount: usize) -> Vec<u16> {
        let probs = &self.data.probs;

        let mut binding = argsort_by(probs, &|a: &f64, b: &f64| a.partial_cmp(b).unwrap());

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

        unreachable!("No full-arena bets found, somehow");
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

    /// Creates a Bets object that consists of all max-TER bets.
    /// This is mostly for debugging purposes.
    pub fn make_all_max_ter_bets(&self) -> Bets {
        let indices = self.max_ter_indices(3124);

        let mut bets = Bets::new(self, indices, None);
        bets.fill_bet_amounts(self);
        bets
    }

    /// Creates a Bets object that consists of the highest ER bets that
    /// are greater than or equal to the given units.
    pub fn make_units_bets(&self, units: u32) -> Option<Bets> {
        let sorted_probs = self.get_sorted_probs_indices(true, 3124);

        let mut units_indices = Vec::<u16>::with_capacity(self.max_amount_of_bets());

        for index in sorted_probs.iter() {
            if self.data.odds[*index as usize] >= units {
                units_indices.push(*index);
                if units_indices.len() == units_indices.capacity() {
                    break;
                }
            }
        }

        if units_indices.is_empty() {
            return None;
        }

        let mut bets = Bets::new(self, units_indices, None);

        bets.fill_bet_amounts(self);

        Some(bets)
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

    /// Creates a Bets object that consists of 10-bets on the selected pirates.
    /// Returns an error if the pirates binary is invalid.
    /// Returns an error if the amount of pirates is invalid.
    /// Returns an error if the amount of pirates is greater than 3.
    /// Returns an error if the amount of pirates is less than 1.
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

    /// Returns the amount of units you'd win if you placed the given bets.
    /// Returns 0 if there are no winners yet.
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

    /// Returns the amount of neopoints you'd win if you placed the given bets.
    /// Returns 0 if there are no winners yet.
    /// Returns 0 if there are no bet amounts.
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

    /// Creates a URL for the given bets.
    pub fn make_url(&self, bets: &Bets, include_domain: bool, all_data: bool) -> String {
        let mut url = String::new();

        if include_domain {
            url.push_str("https://neofood.club");
        }

        let use_15 = self.modifier.is_charity_corner() || bets.len() > 10;
        if use_15 {
            url.push_str("/15");
        }

        url.push_str(&format!("/#round={}", self.round()));

        url.push_str(&format!("&b={}", bets.bets_hash()));

        if let Some(amounts_hash) = bets.amounts_hash() {
            url.push_str(&format!("&a={}", amounts_hash));
        }

        if all_data {
            let mut params = vec![];

            let pirates = serde_json::to_string(&self.round_data.pirates)
                .expect("Failed to serialize pirates.");
            params.push(("pirates", pirates.as_str()));

            let opening_odds = serde_json::to_string(&self.round_data.openingOdds)
                .expect("Failed to serialize openingOdds.");
            params.push(("openingOdds", opening_odds.as_str()));

            let mut params = vec![];

            let current_odds = serde_json::to_string(&self.round_data.currentOdds)
                .expect("Failed to serialize currentOdds.");
            params.push(("currentOdds", current_odds.as_str()));

            let winners_string = if self.is_over() {
                serde_json::to_string(&self.winners()).unwrap()
            } else {
                String::new()
            };
            if !winners_string.is_empty() {
                params.push(("winners", winners_string.as_str()));
            }

            let timestamp = self.timestamp().unwrap_or_default();
            if !timestamp.is_empty() {
                params.push(("timestamp", timestamp.as_str()));
            }

            url.push_str(&stringify(params));
        }

        url
    }

    /// Creates a deep copy of the NeoFoodClub object.
    /// If `model` is None, the model is going to use the default.
    /// If `modifier` is None, the modifier is going to be empty.
    pub fn copy(&self, model: Option<ProbabilityModel>, modifier: Option<Modifier>) -> NeoFoodClub {
        NeoFoodClub::new(self.round_data.clone(), self.bet_amount, model, modifier)
    }
}

fn validate_round_data(round_data: &RoundData) {
    if round_data.round == 0 {
        panic!("Round number must be greater than 0.");
    }

    if round_data.pirates.len() != 5 {
        panic!("Pirates must have 5 arenas.");
    }

    for arena in round_data.pirates.iter() {
        if arena.len() != 4 {
            panic!("Each arena must have 4 pirates.");
        }
    }

    if round_data.currentOdds.len() != 5 {
        panic!("Current odds must have 5 arenas.");
    }

    for arena in round_data.currentOdds.iter() {
        if arena.len() != 5 {
            panic!("Each arena in currentOdds must have 5 integers, first one being 1.");
        }

        for (index, odds) in arena.iter().enumerate() {
            if index == 0 {
                if *odds != 1 {
                    panic!("First integer in each arena in currentOdds must be 1.");
                }
            } else if *odds < 2 || *odds > 13 {
                panic!("Odds must be between 2 and 13.");
            }
        }
    }

    if round_data.openingOdds.len() != 5 {
        panic!("Opening odds must have 5 arenas.");
    }

    for arena in round_data.openingOdds.iter() {
        if arena.len() != 5 {
            panic!("Each arena in openingOdds must have 5 integers, first one being 1.");
        }

        for (index, odds) in arena.iter().enumerate() {
            if index == 0 {
                if *odds != 1 {
                    panic!("First integer in each arena in openingOdds must be 1.");
                }
            } else if *odds < 2 || *odds > 13 {
                panic!("Odds must be between 2 and 13.");
            }
        }
    }

    if round_data.foods.is_some() {
        let foods = round_data.foods.as_ref().unwrap();
        if foods.len() != 5 {
            panic!("Foods must have 5 arenas.");
        }

        for arena in foods.iter() {
            if arena.len() != 10 {
                panic!("Each arena in foods must have 10 integers.");
            }

            for food in arena.iter() {
                if *food < 1 || *food > 40 {
                    panic!("Food integers must be between 1 and 40.");
                }
            }
        }
    }

    if round_data.winners.is_some() {
        let winners = round_data.winners.as_ref().unwrap();
        if winners.len() != 5 {
            panic!("Winners must have 5 integers.");
        }

        // the winners have to either be all 0, or all 1-4, let's check both
        let all_zero = winners.iter().all(|&x| x == 0);
        let all_one_to_four = winners.iter().all(|&x| (1..=4).contains(&x));

        if !(all_zero ^ all_one_to_four) {
            panic!("Winners must either be all 0, or all 1-4.");
        }
    }
}
