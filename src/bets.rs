use crate::{
    math::{
        bet_amounts_to_amounts_hash, bets_hash_to_bet_binaries, bets_hash_value, binary_to_indices,
    },
    nfc::NeoFoodClub,
    odds::Odds,
};

#[derive(Debug, Clone)]
pub struct Bets {
    pub array_indices: Vec<u16>,
    pub amounts: Option<Vec<Option<u32>>>,
    pub odds: Odds,
}

impl Bets {
    pub fn new(nfc: &NeoFoodClub, indices: Vec<u16>, amounts: Option<Vec<Option<u32>>>) -> Self {
        if let Some(amounts) = &amounts {
            if amounts.len() != indices.len() {
                panic!("Bet amounts must be the same length as indices");
            }
        }

        Self {
            array_indices: indices.clone(),
            amounts,
            odds: Odds::new(nfc, indices),
        }
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
        self.amounts = Some(amounts);
    }

    /// Creates a new Bets struct from a list of binaries
    pub fn from_binaries(nfc: &NeoFoodClub, binaries: Vec<u32>) -> Self {
        let bins = &nfc.data.bins;

        let bin_indices: Vec<u16> = bins
            .iter()
            .enumerate()
            .filter_map(|(i, b)| {
                if binaries.contains(b) {
                    Some(i as u16)
                } else {
                    None
                }
            })
            .collect();

        Self::new(nfc, bin_indices, None)
    }

    /// Creates a new Bets struct from a hash
    pub fn from_hash(nfc: &NeoFoodClub, hash: &str) -> Self {
        let binaries = bets_hash_to_bet_binaries(hash);

        Self::from_binaries(nfc, binaries)
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
    pub fn get_indices(&self, nfc: &NeoFoodClub) -> Vec<[u8; 5]> {
        self.array_indices
            .iter()
            .map(|i| binary_to_indices(nfc.data.bins[*i as usize]))
            .collect()
    }

    /// Returns the bet binaries
    pub fn get_binaries(&self, nfc: &NeoFoodClub) -> Vec<u32> {
        self.array_indices
            .iter()
            .map(|i| nfc.data.bins[*i as usize])
            .collect()
    }

    /// Returns a string of the hash of the bets
    pub fn bets_hash(&self, nfc: &NeoFoodClub) -> String {
        bets_hash_value(self.get_indices(nfc))
    }

    /// Returns a string of the hash of the bet amounts, if it can
    pub fn amounts_hash(&self) -> Option<String> {
        self.amounts
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
    pub fn is_crazy(&self, nfc: &NeoFoodClub) -> bool {
        self.array_indices.iter().all(|i| {
            let binary = nfc.data.bins[*i as usize];
            binary.count_ones() == 5
        })
    }

    /// Returns whether or not this set is a "gambit" set.
    /// The rules for what a gambit is, is *somewhat* arbitrary:
    ///     - The largest integer in the binary representation of the bet set must have five 1's.
    ///     - All bets must be subsets of the largest integer.
    ///     - There must be at least 2 bets.
    pub fn is_gambit(&self, nfc: &NeoFoodClub) -> bool {
        if self.array_indices.len() < 2 {
            return false;
        }

        let binaries = self.get_binaries(nfc);

        let highest: u32 = *binaries.iter().max().unwrap();

        if highest.count_ones() != 5 {
            return false;
        }

        binaries.iter().all(|b| (highest & *b) == *b)
    }

    /// Returns whether or not this set is guaranteed to profit.
    /// Must be bustproof, as well.
    pub fn is_guaranteed_win(&self, nfc: &NeoFoodClub) -> bool {
        if !self.is_bustproof() {
            return false;
        }

        let Some(amounts) = &self.amounts else {
            return false;
        };

        // if any amounts are None, return false
        if amounts.iter().any(|a| a.is_none()) {
            return false;
        }

        let highest_bet_amount = amounts.iter().max().unwrap().unwrap_or(0);

        if highest_bet_amount == 0 {
            return false;
        }

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
            .map(|i| nfc.data.odds[*i as usize])
            .collect()
    }
}
