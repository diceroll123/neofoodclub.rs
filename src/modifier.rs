use std::collections::HashMap;

use bitflags::bitflags;
use chrono::NaiveTime;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ModifierFlags: i32 {
        /// No modifiers
        const EMPTY = 0b00000000;

        /// General modifier - Makes max TER use ER instead of NE
        const GENERAL = 0b00000001;

        /// Opening odds modifier - Makes bets use opening odds instead of current odds for calculations
        const OPENING_ODDS = 0b00000010;

        /// Reverse modifier - Makes bets use reverse ER odds for calculations
        const REVERSE = 0b00000100;

        /// Charity Corner modifier - Makes bets use 15 bets instead of 10
        const CHARITY_CORNER = 0b00001000;
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Modifier {
    pub value: i32,
    pub custom_odds: Option<HashMap<u8, u8>>,
    pub custom_time: Option<NaiveTime>,
}

impl Modifier {
    pub fn new(
        value: i32,
        custom_odds: Option<HashMap<u8, u8>>,
        custom_time: Option<NaiveTime>,
    ) -> Self {
        // loop through custom_odds if it's not None and check if the keys are between 1-20 and the values are between 2-13
        if let Some(custom_odds) = custom_odds.clone() {
            for (key, value) in custom_odds.iter() {
                if *key < 1 || *key > 20 {
                    panic!("Invalid pirate ID, need 1-20, got {}", *key);
                }
                if *value < 2 || *value > 13 {
                    panic!("Invalid odds, need 2-13, got {}", *value);
                }
            }
        }

        Self {
            value,
            custom_odds,
            custom_time,
        }
    }
}

impl Modifier {
    // flags

    pub fn is_empty(&self) -> bool {
        ModifierFlags::from_bits(self.value).unwrap().is_empty()
    }

    pub fn is_general(&self) -> bool {
        ModifierFlags::from_bits(self.value)
            .unwrap()
            .contains(ModifierFlags::GENERAL)
    }

    pub fn is_opening_odds(&self) -> bool {
        ModifierFlags::from_bits(self.value)
            .unwrap()
            .contains(ModifierFlags::OPENING_ODDS)
    }

    pub fn is_reverse(&self) -> bool {
        ModifierFlags::from_bits(self.value)
            .unwrap()
            .contains(ModifierFlags::REVERSE)
    }

    pub fn is_charity_corner(&self) -> bool {
        ModifierFlags::from_bits(self.value)
            .unwrap()
            .contains(ModifierFlags::CHARITY_CORNER)
    }
}

impl Modifier {
    /// If the modifier has custom odds or opening odds, this returns true.
    /// Basically, this is a marker to know whether or not this
    /// modifier edits food club data, meaning we will not store it anywhere.
    pub fn modified(&self) -> bool {
        self.custom_odds.is_some() || self.is_opening_odds() || self.custom_time.is_some()
    }
}
