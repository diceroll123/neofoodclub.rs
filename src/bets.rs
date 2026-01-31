use comfy_table::Table;

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
    /// A hash representing variable bet amounts
    AmountHash(String),
    /// A vector of variable bet amounts (can cause errors if length mismatches)
    Amounts(Vec<Option<u32>>),
    /// A single amount applied to all bets (never causes length mismatch errors)
    AllSame(u32),
    /// No bet amounts
    None,
}

impl BetAmounts {
    /// Returns the bet amounts as a vector of Option<u32>
    /// If the BetAmounts is None, returns None
    /// For AllSame, this requires the length parameter
    pub fn to_vec(&self, length: usize) -> Result<Option<Vec<Option<u32>>>, String> {
        match self {
            BetAmounts::AmountHash(hash) => {
                let amounts = amounts_hash_to_bet_amounts(hash)?;
                Ok(Some(Self::clean_amounts(&amounts)))
            }
            BetAmounts::Amounts(amounts) => Ok(Some(Self::clean_amounts(amounts))),
            BetAmounts::AllSame(amount) => {
                if length == 0 {
                    Ok(None)
                } else {
                    let clamped = (*amount).clamp(BET_AMOUNT_MIN, BET_AMOUNT_MAX);
                    Ok(Some(vec![Some(clamped); length]))
                }
            }
            BetAmounts::None => Ok(None),
        }
    }

    /// Creates a new BetAmounts from a single bet amount
    /// This creates an AllSame variant which can never cause length mismatch errors
    pub fn from_amount(amount: u32) -> Self {
        if !(BET_AMOUNT_MIN..=BET_AMOUNT_MAX).contains(&amount) {
            return BetAmounts::None;
        }

        BetAmounts::AllSame(amount)
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
    pub array_indices: Vec<usize>,
    pub bet_binaries: Vec<u32>,
    pub bet_amounts: Option<Vec<Option<u32>>>,
    pub odds: Odds,
}

impl Bets {
    /// Creates a new Bets struct from a list of indices (without bet amounts)
    pub fn new(nfc: &NeoFoodClub, indices: Vec<usize>) -> Self {
        let data = nfc.round_dict_data();
        let mut bet_binaries = Vec::with_capacity(indices.len());
        bet_binaries.extend(indices.iter().map(|&i| data.bins[i]));

        let odds = Odds::new(nfc, &indices);

        Self {
            array_indices: indices,
            bet_binaries,
            bet_amounts: None,
            odds,
        }
    }

    /// Creates a new Bets struct with bet amounts that may fail if lengths don't match
    pub fn try_new(
        nfc: &NeoFoodClub,
        indices: Vec<usize>,
        amounts: BetAmounts,
    ) -> Result<Self, String> {
        let mut bets = Self::new(nfc, indices);
        bets.set_bet_amounts(&Some(amounts))?;
        Ok(bets)
    }

    /// Creates a new Bets struct with a single bet amount for all bets (infallible)
    /// This is a simpler API when you want the same amount for all bets
    pub fn new_with_amount(nfc: &NeoFoodClub, indices: Vec<usize>, amount: Option<u32>) -> Self {
        let mut bets = Self::new(nfc, indices);

        if let Some(amt) = amount {
            bets.set_bet_amount_all_same(amt);
        }

        bets
    }

    /// Sets the bet amounts for the bets
    /// Returns an error if the amounts length doesn't match the bet count
    /// (except for AllSame which always works)
    pub fn set_bet_amounts(&mut self, amounts: &Option<BetAmounts>) -> Result<(), String> {
        let Some(betamount) = amounts else {
            self.bet_amounts = None;
            return Ok(());
        };

        let Some(amounts) = betamount.to_vec(self.array_indices.len())? else {
            self.bet_amounts = None;
            return Ok(());
        };

        // For AllSame variant, length always matches due to to_vec implementation
        // For other variants, check length
        if !matches!(betamount, BetAmounts::AllSame(_)) && amounts.len() != self.array_indices.len()
        {
            return Err(format!(
                "Bet amounts must be the same length as bet indices, or None. Provided: {} Expected: {}",
                amounts.len(), self.array_indices.len()
            ));
        }

        self.bet_amounts = Some(
            amounts
                .iter()
                .map(|x| x.map(|x| x.clamp(BET_AMOUNT_MIN, BET_AMOUNT_MAX)))
                .collect(),
        );

        Ok(())
    }

    /// Sets the bet amounts for the bets (infallible version for AllSame)
    /// This is a convenience method that never returns an error
    fn set_bet_amount_all_same(&mut self, amount: u32) {
        if self.array_indices.is_empty() {
            self.bet_amounts = None;
            return;
        }

        let clamped = amount.clamp(BET_AMOUNT_MIN, BET_AMOUNT_MAX);
        self.bet_amounts = Some(vec![Some(clamped); self.array_indices.len()]);
    }

    /// Returns the net expected value of each bet
    pub fn net_expected_list(&self, nfc: &NeoFoodClub) -> Vec<f64> {
        let Some(amounts) = &self.bet_amounts else {
            return vec![];
        };
        let data = nfc.round_dict_data();

        self.array_indices
            .iter()
            .zip(amounts.iter())
            .map(|(i, a)| {
                let er = data.ers[*i];
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
        let data = nfc.round_dict_data();
        self.array_indices.iter().map(|&i| data.ers[i]).collect()
    }

    /// Returns the sum of expected return of the bets
    pub fn expected_return(&self, nfc: &NeoFoodClub) -> f64 {
        self.expected_return_list(nfc).iter().sum()
    }

    /// Fills the bet amounts in-place with the maximum possible amount to hit 1 million.
    /// In short, for each bet we divide 1_000_000 by the odds, and round up.
    /// Then we use whichever is smaller, the bet amount or the result of that equation.
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

            let amount = bet_amount.min(div).max(BET_AMOUNT_MIN);
            amounts.push(Some(amount));
        }
        self.bet_amounts = Some(amounts);
    }

    /// Creates a new Bets struct from a list of binaries
    pub fn from_binaries(nfc: &NeoFoodClub, binaries: Vec<u32>) -> Self {
        // maintaining the order of the binaries is important, at the cost of some performance
        let data = nfc.round_dict_data();
        let bin_indices: Vec<usize> = binaries
            .iter()
            .filter_map(|b| data.bins.iter().position(|bin| bin == b))
            .collect();

        Self::new(nfc, bin_indices)
    }

    /// Creates a new Bets struct from a hash
    pub fn from_hash(nfc: &NeoFoodClub, hash: &str) -> Result<Self, String> {
        let binaries = bets_hash_to_bet_binaries(hash)?;

        Ok(Self::from_binaries(nfc, binaries))
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
        let mut out = Vec::with_capacity(self.bet_binaries.len());
        out.extend(self.bet_binaries.iter().map(|b| binary_to_indices(*b)));
        out
    }

    /// Returns the bet binaries
    pub fn get_binaries(&self) -> Vec<u32> {
        self.bet_binaries.to_vec()
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
        self.odds.bust().is_none()
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

        self.count_tenbets() > 0
    }

    /// Returns the number of tenbets in this set
    pub fn count_tenbets(&self) -> u32 {
        self.bet_binaries
            .iter()
            .fold(u32::MAX, |acc, &b| acc & b)
            .count_ones()
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

        let highest = match self.bet_binaries.iter().max() {
            Some(&max) if max.count_ones() == 5 => max,
            _ => return false,
        };

        self.bet_binaries.iter().all(|&b| (highest & b) == b)
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

        let highest_bet_amount = amounts.iter().flatten().max().unwrap();

        let lowest_winning_bet_amount = self
            .odds_values(nfc)
            .iter()
            .zip(amounts.iter().flatten())
            .map(|(odds, amount)| odds * amount)
            .min()
            .unwrap();

        highest_bet_amount < &lowest_winning_bet_amount
    }

    /// Returns the odds of the bets
    pub fn odds_values(&self, nfc: &NeoFoodClub) -> Vec<u32> {
        let data = nfc.round_dict_data();
        self.array_indices.iter().map(|i| data.odds[*i]).collect()
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

        let column = table.column_mut(0).expect("Column 0 should exist");
        column.set_cell_alignment(comfy_table::CellAlignment::Right);

        table.to_string()
    }

    /// Returns a table visualization of the bets, with stats
    pub fn stats_table(&self, nfc: &NeoFoodClub) -> String {
        let data = nfc.round_dict_data();
        let mut table = Table::new();

        let mut headers = vec!["#", "Odds", "ER"];

        let nes = self.net_expected_list(nfc);

        if !nes.is_empty() {
            headers.push("NE");
        }

        headers.extend(vec!["MaxBet", "Hex"]);

        headers.extend(ARENA_NAMES);

        table.set_header(headers);

        let arenas = nfc.get_arenas();

        let indices = self.get_indices();
        for (bet_index, (&array_index, bet_indices)) in
            self.array_indices.iter().zip(indices.iter()).enumerate()
        {
            let bet_binary = self.bet_binaries[bet_index];
            let mut row = vec![(bet_index + 1).to_string()];

            let hex = format!("0x{bet_binary:0>5X}");

            row.extend(vec![
                data.odds[array_index].to_string(),
                format!("{:.3}:1", data.ers[array_index]),
            ]);

            if !nes.is_empty() {
                row.push(format!("{:.2}", nes[bet_index]));
            }

            row.extend(vec![data.maxbets[array_index].to_string(), hex]);

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

        let column = table.column_mut(0).expect("Column 0 should exist");
        column.set_cell_alignment(comfy_table::CellAlignment::Right);

        for column in table.column_iter_mut().skip(1) {
            column.set_cell_alignment(comfy_table::CellAlignment::Center);
        }

        table.to_string()
    }
}
