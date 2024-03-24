use comfy_table::Table;
use itertools::Itertools;

use crate::{
    arena::ARENA_NAMES,
    math::{
        amounts_hash_to_bet_amounts, bet_amounts_to_amounts_hash, bets_hash_to_bet_binaries,
        bets_hash_value, binary_to_indices, pirates_binary, BET_AMOUNT_MAX, BET_AMOUNT_MIN,
    },
    nfc::NeoFoodClub,
    odds::Odds,
    pirates::PartialPirateThings,
};

/// A representation of a set of bet amounts
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BetAmounts {
    AmountHash(String),
    Amounts(Vec<Option<u32>>),
    None,
}

impl BetAmounts {
    /// Returns the bet amounts as a vector of Option<u32>
    /// If the BetAmounts is None, returns None
    pub fn to_vec(&self) -> Option<Vec<Option<u32>>> {
        match self {
            BetAmounts::AmountHash(hash) => {
                Some(Self::clean_amounts(&amounts_hash_to_bet_amounts(hash)))
            }
            BetAmounts::Amounts(amounts) => Some(Self::clean_amounts(amounts)),
            BetAmounts::None => None,
        }
    }

    /// Creates a new BetAmounts from a length and bet amount
    pub fn from_amount(amount: u32, length: usize) -> Self {
        if length == 0 {
            return BetAmounts::None;
        }

        if amount > BET_AMOUNT_MAX || amount <= BET_AMOUNT_MIN {
            return BetAmounts::None;
        }

        BetAmounts::Amounts(vec![Some(amount); length])
    }

    /// Creates a new BetAmounts from a vector of optional bet amounts
    fn clean_amounts(amounts: &[Option<u32>]) -> Vec<Option<u32>> {
        let mut cleaned = amounts.to_owned();
        while cleaned.last() == Some(&None) {
            cleaned.pop();
        }
        cleaned
    }
}

/// A container for a set of bets
#[derive(Debug, Clone)]
pub struct Bets {
    pub array_indices: Vec<u16>,
    pub bet_binaries: Vec<u32>,
    pub bet_amounts: Option<Vec<Option<u32>>>,
    pub odds: Odds,
}

impl Bets {
    /// Creates a new Bets struct from a list of indices mapped to the RoundDictData of the NFC object
    pub fn new(nfc: &NeoFoodClub, indices: Vec<u16>, amounts: Option<BetAmounts>) -> Self {
        let mut bets = Self {
            array_indices: indices.clone(),
            bet_binaries: indices
                .iter()
                .map(|i| nfc.round_dict_data().bins[*i as usize])
                .collect(),
            bet_amounts: None,
            odds: Odds::new(nfc, indices),
        };

        bets.set_bet_amounts(&amounts);

        bets
    }

    /// Sets the bet amounts for the bets
    pub fn set_bet_amounts(&mut self, amounts: &Option<BetAmounts>) {
        let Some(betamount) = amounts else {
            self.bet_amounts = None;
            return;
        };

        let Some(amounts) = betamount.to_vec() else {
            self.bet_amounts = None;
            return;
        };

        if amounts.len() != self.array_indices.len() {
            panic!("Bet amounts must be the same length as bet indices, or None. Provided: {} Expected {}", amounts.len(), self.array_indices.len());
        }

        self.bet_amounts = Some(
            amounts
                .iter()
                .map(|x| x.map(|x| x.clamp(BET_AMOUNT_MIN, BET_AMOUNT_MAX)))
                .collect(),
        );
    }

    /// Returns the net expected value of each bet
    pub fn net_expected_list(&self, nfc: &NeoFoodClub) -> Vec<f64> {
        let Some(amounts) = &self.bet_amounts else {
            return vec![];
        };

        self.array_indices
            .iter()
            .zip(amounts.iter())
            .map(|(i, a)| {
                let er = nfc.round_dict_data().ers[*i as usize];
                let amount = a.unwrap_or(0) as f64;
                amount.mul_add(er, -amount)
            })
            .collect()
    }

    /// Returns the sum of net expected value of the bets
    pub fn net_expected(&self, nfc: &NeoFoodClub) -> f64 {
        self.net_expected_list(nfc).iter().sum()
    }

    /// Returns the expected return of each bet
    pub fn expected_return_list(&self, nfc: &NeoFoodClub) -> Vec<f64> {
        self.array_indices
            .iter()
            .map(|i| nfc.round_dict_data().ers[*i as usize])
            .collect()
    }

    /// Returns the sum of expected return of the bets
    pub fn expected_return(&self, nfc: &NeoFoodClub) -> f64 {
        self.expected_return_list(nfc).iter().sum()
    }

    /// Fills the bet amounts in-place with the maximum possible amount to hit 1 million.
    /// In short, for each bet we divide 1_000_000 by the odds, and round up.
    /// Then we use whichever is smaller, the bet amount or the result of that equation.
    /// If the result is less than 50, we use 50 instead.
    pub fn fill_bet_amounts(&mut self, nfc: &NeoFoodClub) {
        let Some(bet_amount) = nfc.bet_amount else {
            return;
        };

        let mut amounts = Vec::<Option<u32>>::with_capacity(self.array_indices.len());
        for odds in self.odds_values(nfc).iter() {
            let mut div = 1_000_000 / odds;
            let modulo = 1_000_000 % odds;

            if modulo > 0 {
                div += 1;
            }

            let amount = bet_amount.min(div).max(50);
            amounts.push(Some(amount));
        }
        self.bet_amounts = Some(amounts);
    }

    /// Creates a new Bets struct from a list of binaries
    pub fn from_binaries(nfc: &NeoFoodClub, binaries: Vec<u32>) -> Self {
        // maintaining the order of the binaries is important, at the cost of some performance
        let bin_index_map: std::collections::HashMap<u32, u16> = nfc
            .round_dict_data()
            .bins
            .iter()
            .enumerate()
            .map(|(i, &bin)| (bin, i as u16))
            .collect();

        let bin_indices: Vec<u16> = binaries
            .iter()
            .filter_map(|b| bin_index_map.get(b))
            .cloned()
            .collect();

        let unique_bin_indices: Vec<u16> = bin_indices.into_iter().unique().collect();

        Self::new(nfc, unique_bin_indices, None)
    }

    /// Creates a new Bets struct from a hash
    pub fn from_hash(nfc: &NeoFoodClub, hash: &str) -> Self {
        let binaries = bets_hash_to_bet_binaries(hash);

        Self::from_binaries(nfc, binaries)
    }

    /// Creates a new Bets struct from pirate indices
    pub fn from_indices(nfc: &NeoFoodClub, indices: Vec<[u8; 5]>) -> Self {
        let bins: Vec<u32> = indices.iter().map(|i| pirates_binary(*i)).collect();

        Self::from_binaries(nfc, bins)
    }

    /// Returns the number of bets
    pub fn len(&self) -> usize {
        self.array_indices.len()
    }

    /// Whether or not there are any bets
    pub fn is_empty(&self) -> bool {
        self.array_indices.is_empty()
    }

    /// Returns a nested array of the indices of the pirates in their arenas
    /// making up these bets.
    pub fn get_indices(&self) -> Vec<[u8; 5]> {
        self.bet_binaries
            .iter()
            .map(|b| binary_to_indices(*b))
            .collect()
    }

    /// Returns the bet binaries
    pub fn get_binaries(&self) -> &Vec<u32> {
        &self.bet_binaries
    }

    /// Returns a string of the hash of the bets
    pub fn bets_hash(&self) -> String {
        bets_hash_value(self.get_indices())
    }

    /// Returns a string of the hash of the bet amounts, if it can
    pub fn amounts_hash(&self) -> Option<String> {
        self.bet_amounts
            .as_ref()
            .map(|amounts| bet_amounts_to_amounts_hash(amounts))
    }

    /// Returns whether or not this set is capable of busting
    /// if there are no odds, returns None
    pub fn is_bustproof(&self) -> bool {
        self.odds.bust.is_none()
    }

    /// Returns whether or not this set is "crazy"
    /// as in, all bets have filled arenas
    pub fn is_crazy(&self) -> bool {
        self.bet_binaries.iter().all(|bin| bin.count_ones() == 5)
    }

    /// Returns whether or not this set is a "tenbet" set.
    /// A tenbet set is a set of bets where all bets have at least one pirate in common.
    pub fn is_tenbet(&self) -> bool {
        if self.bet_binaries.len() < 10 {
            return false;
        }

        let anded = self
            .bet_binaries
            .iter()
            .fold(None, |acc, &b| {
                if let Some(result) = acc {
                    Some(result & b)
                } else {
                    Some(b)
                }
            })
            .unwrap()
            .count_ones();

        anded > 0
    }

    /// Returns whether or not this set is a "gambit" set.
    /// The rules for what a gambit is, is *somewhat* arbitrary:
    ///     - The largest integer in the binary representation of the bet set must have five 1's.
    ///     - All bets must be subsets of the largest integer.
    ///     - There must be at least 2 bets.
    pub fn is_gambit(&self) -> bool {
        if self.array_indices.len() < 2 {
            return false;
        }

        let highest: u32 = *self.bet_binaries.iter().max().unwrap();

        if highest.count_ones() != 5 {
            return false;
        }

        self.bet_binaries.iter().all(|b| (highest & *b) == *b)
    }

    /// Returns whether or not this set is guaranteed to profit.
    /// Must be bustproof, as well.
    pub fn is_guaranteed_win(&self, nfc: &NeoFoodClub) -> bool {
        if !self.is_bustproof() {
            return false;
        }

        let Some(amounts) = &self.bet_amounts else {
            return false;
        };

        // if any amounts are None, return false
        if amounts.iter().any(|a| a.is_none()) {
            return false;
        }

        let highest_bet_amount = amounts.iter().max().unwrap().unwrap();

        // multiply each odds by each bet amount
        let lowest_winning_bet_amount = self
            .odds_values(nfc)
            .iter()
            .enumerate()
            .map(|(index, odds)| odds * amounts[index].unwrap())
            .min()
            .unwrap();

        highest_bet_amount < lowest_winning_bet_amount
    }

    /// Returns the odds of the bets
    pub fn odds_values(&self, nfc: &NeoFoodClub) -> Vec<u32> {
        self.array_indices
            .iter()
            .map(|i| nfc.round_dict_data().odds[*i as usize])
            .collect()
    }

    /// Makes a URL for the bets using the NeoFoodClub object
    pub fn make_url(&self, nfc: &NeoFoodClub, include_domain: bool, all_data: bool) -> String {
        nfc.make_url(Some(self), include_domain, all_data)
    }

    /// Returns a table visualization of the bets
    pub fn table(&self, nfc: &NeoFoodClub) -> String {
        let mut table = Table::new();

        let mut headers = vec!["#"];

        headers.extend(ARENA_NAMES);

        table.set_header(headers);

        let arenas = nfc.get_arenas();

        for (bet_index, bet_row) in self.get_indices().iter().enumerate() {
            let mut row = vec![(bet_index + 1).to_string()];

            for (arena_index, pirate_index) in bet_row.iter().enumerate() {
                if pirate_index == &0 {
                    row.push("".to_string());
                } else {
                    let arena = arenas.get_arena(arena_index).unwrap();
                    let pirate = &arena.get_pirate_by_index(pirate_index - 1).unwrap();
                    row.push(pirate.get_name().to_string());
                }
            }
            table.add_row(row);
        }

        table.to_string()
    }

    /// Returns a table visualization of the bets, with stats
    pub fn stats_table(&self, nfc: &NeoFoodClub) -> String {
        let mut table = Table::new();

        let mut headers = vec!["#", "Odds", "ER", "MaxBet", "Hex"];

        headers.extend(ARENA_NAMES);

        table.set_header(headers);

        let arenas = nfc.get_arenas();

        for (bet_index, (bet_binary, bet_indices)) in self
            .get_binaries()
            .iter()
            .zip(self.get_indices().iter())
            .enumerate()
        {
            let mut row = vec![(bet_index + 1).to_string()];

            let bin_index = nfc
                .round_dict_data()
                .bins
                .iter()
                .position(|&r| r == *bet_binary)
                .unwrap();

            row.extend(vec![
                nfc.round_dict_data().odds[bin_index].to_string(),
                format!("{:.3}:1", nfc.round_dict_data().ers[bin_index]),
                nfc.round_dict_data().maxbets[bin_index].to_string(),
                format!("0x{:X}", bet_binary),
            ]);

            for (arena_index, pirate_index) in bet_indices.iter().enumerate() {
                if pirate_index == &0 {
                    row.push("".to_string());
                } else {
                    let arena = arenas.get_arena(arena_index).unwrap();
                    let pirate = &arena.get_pirate_by_index(pirate_index - 1).unwrap();
                    row.push(pirate.get_name().to_string());
                }
            }
            table.add_row(row);
        }

        table.to_string()
    }
}
