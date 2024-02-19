use crate::{
    math::{bet_amounts_to_amounts_hash, bets_hash_value, binary_to_indices},
    nfc::NeoFoodClub,
    odds::Odds,
};

#[derive(Debug, Clone)]
pub struct Bets<'a> {
    pub nfc: &'a NeoFoodClub,
    pub array_indices: Vec<u16>,
    pub amounts: Option<Vec<u32>>,
    pub odds: Odds,
}

impl<'a> Bets<'a> {
    pub fn new(nfc: &'a NeoFoodClub, indices: Vec<u16>, amounts: Option<Vec<u32>>) -> Self {
        if let Some(amounts) = &amounts {
            if amounts.len() != indices.len() {
                panic!("Bet amounts must be the same length as indices");
            }
        }

        Self {
            nfc,
            array_indices: indices.clone(),
            amounts,
            odds: Odds::new(nfc, indices),
        }
    }

    pub fn fill_bet_amounts(&mut self) {
        let Some(bet_amount) = self.nfc.bet_amount else {
            return;
        };

        let mut amounts = Vec::<u32>::with_capacity(self.array_indices.len());
        for odds in self.odds_values().iter() {
            let mut div = 1_000_000 / odds;
            let modulo = 1_000_000 % odds;

            if modulo > 0 {
                div += 1;
            }

            let amount = bet_amount.min(div).max(50);
            amounts.push(amount);
        }
        self.amounts = Some(amounts);
    }

    pub fn from_binaries(nfc: &'a NeoFoodClub, binaries: Vec<u32>) -> Self {
        let bins = &nfc.data.bins;

        let bin_indices: Vec<u16> = bins
            .indexed_iter()
            .filter(|(_, b)| binaries.contains(b))
            .map(|(i, _)| i as u16)
            .collect();

        Self::new(nfc, bin_indices, None)
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
    pub fn get_indices(&self) -> Vec<Vec<u8>> {
        self.array_indices
            .iter()
            .map(|i| binary_to_indices(self.nfc.data.bins[*i as usize]))
            .collect()
    }

    /// Returns the bet binaries
    pub fn get_binaries(&self) -> Vec<u32> {
        self.array_indices
            .iter()
            .map(|i| self.nfc.data.bins[*i as usize])
            .collect()
    }

    /// Returns a string of the hash of the bets
    pub fn bets_hash(&self) -> String {
        bets_hash_value(self.get_indices())
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
    pub fn is_crazy(&self) -> bool {
        self.array_indices.iter().all(|i| {
            let binary = self.nfc.data.bins[*i as usize];
            binary.count_ones() == 5
        })
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

        let binaries = self.get_binaries();

        let highest: u32 = *binaries.iter().max().unwrap();

        if highest.count_ones() != 5 {
            return false;
        }

        binaries.iter().all(|b| (highest & *b) == *b)
    }

    /// Returns whether or not this set is guaranteed to profit.
    /// Must be bustproof, as well.
    pub fn is_guaranteed_win(&self) -> bool {
        if !self.is_bustproof() {
            return false;
        }

        let Some(amounts) = &self.amounts else {
            return false;
        };

        let highest_bet_amount = *amounts.iter().max().unwrap();

        // multiply each odds by each bet amount
        let lowest_winning_bet_amount = self
            .odds_values()
            .iter()
            .enumerate()
            .map(|(index, odds)| odds * amounts[index])
            .min()
            .unwrap();

        highest_bet_amount < lowest_winning_bet_amount
    }

    /// Returns the odds of the bets
    pub fn odds_values(&self) -> Vec<u32> {
        self.array_indices
            .iter()
            .map(|i| self.nfc.data.odds[*i as usize])
            .collect()
    }
}
