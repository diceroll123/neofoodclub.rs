use crate::{
    food_adjustments::{NEGATIVE_FOOD, POSITIVE_FOOD},
    math::pirate_binary,
    nfc::NeoFoodClub,
};

pub const PIRATE_NAMES: [&str; 20] = [
    "Dan",
    "Sproggie",
    "Orvinn",
    "Lucky",
    "Edmund",
    "Peg Leg",
    "Bonnie",
    "Puffo",
    "Stuff",
    "Squire",
    "Crossblades",
    "Stripey",
    "Ned",
    "Fairfax",
    "Gooblah",
    "Franchisco",
    "Federismo",
    "Blackbeard",
    "Buck",
    "Tailhook",
];

pub trait PartialPirateThings {
    fn get_name(&self) -> &'static str;
    fn get_image(&self) -> String;
}

pub struct PartialPirate {
    pub id: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Pirate {
    /// The pirate's ID.
    pub id: u8,

    /// The index of the arena in which the pirate is competing.
    pub arena_id: u8,

    /// The index of the pirate in the arena. One-indexed.
    pub index: u8,

    /// The pirate's current odds. 2-13 inclusive.
    pub current_odds: u8,

    /// The pirate's opening odds. 2-13 inclusive.
    pub opening_odds: u8,

    /// The pirate's positive food adjustment.
    pub pfa: Option<u8>,

    /// The pirate's negative food adjustment.
    pub nfa: Option<i8>,

    /// The pirate's total food adjustment. (pfa - nfa)
    pub fa: Option<i8>,

    /// Whether or not the pirate is a winner.
    pub is_winner: bool,
}

impl Pirate {
    /// The pirate's bet-binary representation.
    pub fn binary(&self) -> u32 {
        pirate_binary(self.index + 1, self.arena_id)
    }

    /// The pirates positive foods for a given NFC object.
    pub fn positive_foods(&self, nfc: &NeoFoodClub) -> Option<Vec<u8>> {
        if let Some(nfc_foods) = nfc.foods() {
            let foods: Vec<u8> = nfc_foods[self.arena_id as usize]
                .iter()
                .filter(|&food| POSITIVE_FOOD[self.id as usize - 1][*food as usize - 1] != 0)
                .copied()
                .collect();

            if !foods.is_empty() {
                return Some(foods);
            }
        }
        None
    }

    /// The pirates negative foods for a given NFC object.
    pub fn negative_foods(&self, nfc: &NeoFoodClub) -> Option<Vec<u8>> {
        if let Some(nfc_foods) = nfc.foods() {
            let foods: Vec<u8> = nfc_foods[self.arena_id as usize]
                .iter()
                .filter(|&food| NEGATIVE_FOOD[self.id as usize - 1][*food as usize - 1] != 0)
                .copied()
                .collect();

            if !foods.is_empty() {
                return Some(foods);
            }
        }
        None
    }
}

macro_rules! impl_partial_pirate_things {
    ($($t:ty),+) => {
        $(impl PartialPirateThings for $t {
            /// The pirate's name.
            fn get_name(&self) -> &'static str {
                PIRATE_NAMES[(self.id - 1) as usize]
            }

            /// The pirate's image URL.
            fn get_image(&self) -> String {
                format!(
                    "http://images.neopets.com/pirates/fc/fc_pirate_{}.gif",
                    self.id
                )
            }
        })+
    };
}

impl_partial_pirate_things!(PartialPirate, Pirate);
