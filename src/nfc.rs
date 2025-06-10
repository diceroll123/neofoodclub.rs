use std::cell::OnceCell;
use std::collections::HashSet;

use crate::arena::Arenas;
use crate::bets::Bets;
use crate::math::{
    make_round_dicts, pirates_binary, random_full_pirates_binary, RoundDictData, BET_AMOUNT_MAX,
    BET_AMOUNT_MIN, BIT_MASKS,
};
use crate::modifier::{Modifier, ModifierFlags};
use crate::oddschange::OddsChange;
use crate::round_data::RoundData;
use crate::utils::{argsort_by, get_dst_offset};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use itertools::Itertools;
use querystring::stringify;
use rand::seq::IteratorRandom;
use serde::Deserialize;

use crate::models::multinomial_logit::MultinomialLogitModel;
use crate::models::original::OriginalModel;
use crate::pirates::Pirate;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct RoundDataRaw {
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
    pub modifier: Modifier,
    pub probability_model: ProbabilityModel,
    arenas: OnceCell<Arenas>,
    stds: OnceCell<[[f64; 5]; 5]>,
    data: OnceCell<RoundDictData>,
    max_ter_indices: OnceCell<Vec<usize>>,
    net_expected_indices: OnceCell<Vec<f64>>,
    clamped_max_bets: OnceCell<Vec<u32>>,
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

        use_modifier.apply(&mut round_data);

        let mut nfc = NeoFoodClub {
            round_data,
            bet_amount: None,
            modifier: use_modifier,
            probability_model: model.unwrap_or_default(),
            arenas: OnceCell::new(),
            stds: OnceCell::new(),
            data: OnceCell::new(),
            max_ter_indices: OnceCell::new(),
            net_expected_indices: OnceCell::new(),
            clamped_max_bets: OnceCell::new(),
        };

        nfc.set_bet_amount(bet_amount);

        nfc
    }

    /// Sets the bet amount
    pub fn set_bet_amount(&mut self, amount: Option<u32>) {
        self.bet_amount = amount.map(|x| x.clamp(BET_AMOUNT_MIN, BET_AMOUNT_MAX));
        self.clamped_max_bets = OnceCell::new();
    }

    /// Lazy loads the Arenas object.
    pub fn get_arenas(&self) -> &Arenas {
        self.arenas.get_or_init(|| Arenas::new(&self.round_data))
    }

    /// Lazy loads the probabilities.
    pub fn probabilities(&self) -> [[f64; 5]; 5] {
        *self.stds.get_or_init(|| match self.probability_model {
            ProbabilityModel::OriginalModel => OriginalModel::new(&self.round_data),
            ProbabilityModel::MultinomialLogitModel => {
                MultinomialLogitModel::new(self.get_arenas())
            }
        })
    }

    /// Lazy loads the RoundDictData object.
    pub fn round_dict_data(&self) -> &RoundDictData {
        self.data
            .get_or_init(|| make_round_dicts(self.probabilities(), self.custom_odds()))
    }

    /// Clear our lazy-loaded caches.
    pub fn clear_caches(&mut self) {
        self.arenas = OnceCell::new();
        self.stds = OnceCell::new();
        self.data = OnceCell::new();
        self.clamped_max_bets = OnceCell::new();
        self.max_ter_indices = OnceCell::new();
        self.net_expected_indices = OnceCell::new();
        self.round_data.customOdds = None;
    }

    /// changes the modifier of this NeoFoodClub object
    /// if the modifier is different enough, we clear the caches
    pub fn with_modifier(&mut self, modifier: Modifier) -> &NeoFoodClub {
        let current_modifier = &self.modifier;

        if self.modified()
            || (current_modifier.custom_odds != modifier.custom_odds
                || current_modifier.custom_time != modifier.custom_time
                || current_modifier.is_opening_odds() != modifier.is_opening_odds())
        {
            self.clear_caches();
        }

        self.round_data.customOdds = None;

        self.modifier = modifier;
        self.modifier.apply(&mut self.round_data);
        self
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
            use_modifier.custom_odds,
            use_modifier.custom_time,
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
            customOdds: None,
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

        Some(self.get_arenas().get_pirates_from_binary(bin))
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
    pub fn start(&self) -> &Option<String> {
        &self.round_data.start
    }

    /// Returns the start time of the round in NST.
    /// If the start time is not available, returns None.
    pub fn start_nst(&self) -> Option<DateTime<Tz>> {
        self.round_data.start_nst()
    }

    /// Returns the start time of the round in UTC.
    /// If the start time is not available, returns None.
    pub fn start_utc(&self) -> Option<DateTime<Utc>> {
        self.round_data.start_utc()
    }

    /// Returns the current odds.
    pub fn current_odds(&self) -> &[[u8; 5]; 5] {
        &self.round_data.currentOdds
    }

    /// Returns the custom odds.
    /// If the custom odds are not available, returns the current odds.
    /// Custom odds is just the resolved changes of a Modifier.
    /// Effectively, this is what we use for calculations.
    pub fn custom_odds(&self) -> [[u8; 5]; 5] {
        self.round_data.customOdds.unwrap_or(*self.current_odds())
    }

    /// Returns the opening odds.
    pub fn opening_odds(&self) -> [[u8; 5]; 5] {
        self.round_data.openingOdds
    }

    /// Returns the timestamp of the round in ISO 8601 format as a string.
    pub fn timestamp(&self) -> &Option<String> {
        &self.round_data.timestamp
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
    pub fn changes(&self) -> &Option<Vec<OddsChange>> {
        &self.round_data.changes
    }

    /// Returns the last change of the round in ISO 8601 format as a string.
    /// If the last change is not available, returns None.
    pub fn last_change(&self) -> &Option<String> {
        &self.round_data.lastChange
    }

    /// Returns the last change of the round in NST.
    /// If the last change is not available, returns None.
    pub fn last_change_nst(&self) -> Option<DateTime<Tz>> {
        self.round_data.last_change_nst()
    }

    /// Returns the last change of the round in UTC.
    /// If the last change is not available, returns None.
    pub fn last_change_utc(&self) -> Option<DateTime<Utc>> {
        self.round_data.last_change_utc()
    }

    /// Returns the foods of the round.
    /// If the foods are not available, returns None.
    pub fn foods(&self) -> Option<[[u8; 10]; 5]> {
        self.round_data.foods
    }

    /// Returns whether or not the modifier has made changes to the round data.
    /// We use this to determine if we need to recalculate everything
    /// between
    pub fn modified(&self) -> bool {
        self.custom_odds() != *self.current_odds()
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

    /// Returns the maximum TER values we'll use.
    pub fn max_ters(&self) -> &Vec<f64> {
        let general = self.modifier.is_general();

        if !general && self.bet_amount.is_some() {
            let bet_amount = self.bet_amount.unwrap();

            // if there's a bet amount, we use Net Expected instead of Expected Return
            let maxbets: &Vec<u32> = self.clamped_max_bets.get_or_init(|| {
                self.round_dict_data()
                    .maxbets
                    .iter()
                    .map(|&x| x.max(BET_AMOUNT_MIN).min(bet_amount))
                    .collect()
            });

            let new_ers: &Vec<f64> = self.net_expected_indices.get_or_init(|| {
                maxbets
                    .iter()
                    .zip(self.round_dict_data().ers.iter())
                    .map(|(maxbet, er)| {
                        let mb = *maxbet as f64;
                        mb * er - mb
                    })
                    .collect()
            });
            new_ers
        } else {
            &self.round_dict_data().ers
        }
    }

    /// Returns max-TER indices.
    fn max_ter_indices(&self) -> Vec<usize> {
        let use_ers = self.max_ters();

        let mut binding = argsort_by(use_ers, &|a: &f64, b: &f64| a.total_cmp(b));

        let reverse = self.modifier.is_reverse();
        // since it's reversed to begin with, we reverse it if
        // the modifier does not have the reverse flag
        if !reverse {
            binding.reverse();
        }

        binding
    }

    /// Returns sorted indices of odds
    /// If `descending` is true, returns highest to lowest.
    /// If `descending` is false, returns lowest to highest.
    fn get_sorted_odds_indices(&self, descending: bool, amount: usize) -> Vec<usize> {
        let odds = &self.round_dict_data().odds;

        let mut indices = argsort_by(odds, &|a: &u32, b: &u32| a.cmp(b));

        if descending {
            indices.reverse();
        }

        indices.into_iter().take(amount).collect()
    }

    /// Returns sorted indices of probabilities
    /// If `descending` is true, returns highest to lowest.
    /// If `descending` is false, returns lowest to highest.
    fn get_sorted_probs_indices(&self, descending: bool, amount: usize) -> Vec<usize> {
        let probs = &self.round_dict_data().probs;

        let mut indices = argsort_by(probs, &|a: &f64, b: &f64| a.partial_cmp(b).unwrap());

        if descending {
            indices.reverse();
        }

        indices.into_iter().take(amount).collect()
    }

    /// Return the binary representation of the highest expected return full-arena bet.
    fn get_highest_er_full_bet(&self) -> u32 {
        let max_ter_indices = self.max_ter_indices();

        let index = max_ter_indices
            .into_iter()
            .find(|&index| self.round_dict_data().bins[index].count_ones() == 5)
            .unwrap();

        self.round_dict_data().bins[index]
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
        let indices = self.max_ter_indices();

        let mut bets = Bets::new(self, indices.to_vec(), None);
        bets.fill_bet_amounts(self);
        bets
    }

    /// Creates a Bets object that consists of the highest ER bets that
    /// are greater than or equal to the given units.
    pub fn make_units_bets(&self, units: u32) -> Option<Bets> {
        let sorted_probs = self.get_sorted_probs_indices(true, 3124);

        let mut units_indices = Vec::<usize>::with_capacity(self.max_amount_of_bets());

        for index in sorted_probs.iter() {
            if self.round_dict_data().odds[*index] >= units {
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
        let mut rng = rand::rng();

        let chosen_values: Vec<usize> =
            (0..3124).choose_multiple(&mut rng, self.max_amount_of_bets());

        let mut bets = Bets::new(self, chosen_values, None);
        bets.fill_bet_amounts(self);
        bets
    }

    /// Creates a Bets object that consists of max-TER bets.
    pub fn make_max_ter_bets(&self) -> Bets {
        let indices = self
            .max_ter_indices()
            .iter()
            .take(self.max_amount_of_bets())
            .cloned()
            .collect();

        let mut bets = Bets::new(self, indices, None);
        bets.fill_bet_amounts(self);
        bets
    }

    /// Creates a Bets object that consists of a gambit of the given 5-bet pirates binary.
    pub fn make_gambit_bets(&self, pirates_binary: u32) -> Bets {
        assert_eq!(
            pirates_binary.count_ones(),
            5,
            "Pirates binary must have 5 pirates."
        );

        // get indices of all bets that contain the pirates in the pirates_binary
        let bins = &self.round_dict_data().bins;
        let indices = self
            .get_sorted_odds_indices(true, 3124)
            .into_iter()
            .filter(|&index| bins[index] & pirates_binary == bins[index])
            .take(self.max_amount_of_bets())
            .collect();

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
        self.make_gambit_bets(random_full_pirates_binary())
    }

    /// Creates a Bets object that consits of "crazy" bets.
    /// Crazy bets consist of randomly-selected, full-arena bets.
    /// Following these bets is not recommended.
    pub fn make_crazy_bets(&self) -> Bets {
        let mut binaries: HashSet<u32> = HashSet::with_capacity(self.max_amount_of_bets());

        while binaries.len() < binaries.capacity() {
            binaries.insert(random_full_pirates_binary());
        }

        let mut bets = Bets::from_binaries(self, binaries.into_iter().collect());
        bets.fill_bet_amounts(self);
        bets
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn bustproof_unreachable() -> ! {
        unreachable!("This should never happen.");
    }

    /// Creates a Bets object that consists of bustproof bets.
    /// Returns None if there are no positive arenas.
    pub fn make_bustproof_bets(&self) -> Option<Bets> {
        let positives = self.get_arenas().positives();

        let bets = match positives.len() {
            0 => None,
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

                let best_in_best_arena = best_arena.best();

                let best_pirate_binary = best_in_best_arena[0].binary();

                let binaries: Vec<u32> = best_in_best_arena[1..]
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

                let best_in_best_arena = best_arena.best();
                let best_in_second_best_arena = second_best_arena.best();

                let best_pirate_binary = best_in_best_arena[0].binary();
                let second_best_pirate_binary = best_in_second_best_arena[0].binary();

                let binaries: Vec<u32> = best_in_best_arena[1..]
                    .iter()
                    .map(|pirate| pirate.binary())
                    .chain(
                        best_in_second_best_arena[1..]
                            .iter()
                            .map(|pirate| pirate.binary() | best_pirate_binary),
                    )
                    .chain(third_best_arena.pirates.iter().map(|pirate| {
                        pirate.binary() | best_pirate_binary | second_best_pirate_binary
                    }))
                    .collect();

                Some(Bets::from_binaries(self, binaries))
            }
            _ => Self::bustproof_unreachable(),
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
        let amount_of_pirates = BIT_MASKS
            .iter()
            .map(|mask| (pirates_binary & mask).count_ones())
            .inspect(|&arena_pirates| {
                if arena_pirates > 1 {
                    panic!("You can only pick 1 pirate per arena.");
                }
            })
            .sum::<u32>();

        match amount_of_pirates {
            0 => return Err("You must pick at least 1 pirate, and at most 3.".to_string()),
            1..=3 => (),
            _ => return Err("You must pick 3 pirates at most.".to_string()),
        }

        let bins = self
            .max_ter_indices()
            .iter()
            .map(|&index| self.round_dict_data().bins[index])
            .filter(|&bin| bin & pirates_binary == pirates_binary)
            .take(self.max_amount_of_bets())
            .collect();

        let mut bets = Bets::from_binaries(self, bins);

        bets.fill_bet_amounts(self);

        Ok(bets)
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

    /// Creates a Bets object from a vector of indices.
    /// Unlike the other usages of indices, this one uses the index of our RoundData struct.
    /// For when we do the sorting in Python.
    pub fn make_bets_from_array_indices(&self, array_indices: Vec<usize>) -> Bets {
        let binaries = array_indices
            .iter()
            .map(|&i| self.round_dict_data().bins[i])
            .collect();

        let mut bets = Bets::from_binaries(self, binaries);

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

        bets.array_indices
            .iter()
            .map(|i| {
                let bet_bin = self.round_dict_data().bins[*i];

                if bet_bin & winners_binary == bet_bin {
                    self.round_dict_data().odds[*i]
                } else {
                    0
                }
            })
            .sum()
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

        bets.array_indices
            .iter()
            .enumerate()
            .fold(0, |acc, (bet_index, array_index)| {
                let bet_bin = self.round_dict_data().bins[*array_index];
                if bet_bin & winners_binary == bet_bin {
                    acc + (self.round_dict_data().odds[*array_index]
                        * bet_amounts[bet_index].unwrap_or(0))
                    .clamp(0, 1_000_000)
                } else {
                    acc
                }
            })
    }
}

impl NeoFoodClub {
    // URL-related stuff

    /// Creates a URL for the given bets.
    pub fn make_url(&self, bets: Option<&Bets>, include_domain: bool, all_data: bool) -> String {
        let mut url = String::new();

        if include_domain {
            url.push_str("https://neofood.club");
        }

        let use_15 = self.modifier.is_charity_corner() || bets.is_some_and(|b| b.len() > 10);
        if use_15 {
            url.push_str("/15");
        }

        url.push_str(&format!("/#round={}", self.round()));

        if let Some(bets) = bets {
            url.push_str(&format!("&b={}", bets.bets_hash()));

            if let Some(amounts_hash) = bets.amounts_hash() {
                url.push_str(&format!("&a={amounts_hash}"));
            }
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

            if let Some(timestamp) = self.timestamp().as_ref() {
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
        let mut round_data = self.round_data.clone();
        round_data.customOdds = None;
        NeoFoodClub::new(round_data, self.bet_amount, model, modifier)
    }
}

fn validate_round_data(round_data: &RoundData) {
    if round_data.round == 0 {
        panic!("Round number must be greater than 0.");
    }

    let mut pirate_ids = Vec::<u8>::with_capacity(20);

    for arena in round_data.pirates.iter() {
        for pirate in arena.iter() {
            if pirate_ids.contains(pirate) {
                panic!("Pirates must be unique.");
            }
            if !(&1..=&20).contains(&pirate) {
                panic!("Pirate IDs must be between 1 and 20.");
            }
            pirate_ids.push(*pirate);
        }
    }

    for arena in round_data.currentOdds.iter() {
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

    for arena in round_data.openingOdds.iter() {
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
        for arena in foods.iter() {
            for food in arena.iter() {
                if *food < 1 || *food > 40 {
                    panic!("Food integers must be between 1 and 40.");
                }
            }
        }
    }

    if round_data.winners.is_some() {
        let winners = round_data.winners.as_ref().unwrap();

        // the winners have to either be all 0, or all 1-4, let's check both
        let all_zero = winners.iter().all(|&x| x == 0);
        let all_one_to_four = winners.iter().all(|&x| (1..=4).contains(&x));

        if !(all_zero ^ all_one_to_four) {
            panic!("Winners must either be all 0, or all 1-4.");
        }
    }
}
