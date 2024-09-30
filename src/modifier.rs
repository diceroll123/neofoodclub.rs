use std::collections::HashMap;

use bitflags::bitflags;
use chrono::NaiveTime;
use chrono_tz::US::Pacific;

use crate::round_data::RoundData;

bitflags! {
    /// A set of flags for modifiers.
    /// Each one affects the way certain bets are calculated.
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

/// A struct to represent a modifier.
///
/// A modifier is a set of flags that affect the way certain bets are calculated,
/// as well as custom odds and custom time.
/// Custom odds is a map of pirate IDs to odds.
/// Custom time is in NST. When set, this will change the current odds to the opening odds,
/// and then apply the odds changes up to the custom time, as if making the bets at that time.
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
        if let Some(custom_odds) = custom_odds.as_ref() {
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

    /// Returns true if the modifier has no flags.
    pub fn is_empty(&self) -> bool {
        self.value == 0
    }

    /// Returns true if the modifier has the general flag.
    /// This makes max TER use ER instead of NE.
    pub fn is_general(&self) -> bool {
        self.value & ModifierFlags::GENERAL.bits() != 0
    }

    /// Returns true if the modifier has the opening odds flag.
    pub fn is_opening_odds(&self) -> bool {
        self.value & ModifierFlags::OPENING_ODDS.bits() != 0
    }

    /// Returns true if the modifier has the reverse flag.
    /// This makes bets use reverse ER odds for calculations.
    pub fn is_reverse(&self) -> bool {
        self.value & ModifierFlags::REVERSE.bits() != 0
    }

    /// Returns true if the modifier has the charity corner flag.
    /// This makes bets use 15 bets instead of 10.
    pub fn is_charity_corner(&self) -> bool {
        self.value & ModifierFlags::CHARITY_CORNER.bits() != 0
    }
}

impl Modifier {
    /// Applies the modifier to the round data and returns a new round data object.
    pub fn apply(&self, round_data: &mut RoundData) {
        // first, apply opening odds to current odds if necessary
        if self.is_opening_odds() {
            round_data.customOdds = Some(round_data.openingOdds);
        }

        // apply custom time if necessary
        // only can if start is Some, and custom_time is Some, and changes is Some
        if let Some(start_time_as_nst) = &round_data.start_nst() {
            if let Some(custom_time) = &self.custom_time {
                if let Some(changes) = &round_data.changes {
                    let mut temp_odds = round_data.openingOdds; // as a starting point

                    let mut custom_time = start_time_as_nst
                        .date_naive()
                        .and_time(*custom_time)
                        .and_local_timezone(Pacific)
                        .unwrap();

                    // if the custom time is before the start time, we need to add a day
                    if custom_time < *start_time_as_nst {
                        custom_time += chrono::Duration::try_days(1).unwrap();
                    }

                    let new_changes = changes
                        .iter()
                        .filter(|change| change.timestamp_nst() <= custom_time)
                        .cloned()
                        .collect::<Vec<_>>();

                    if !new_changes.is_empty() {
                        for change in new_changes.iter() {
                            temp_odds[change.arena_index()][change.pirate_index()] = change.new;
                        }

                        round_data.changes = Some(new_changes);
                    } else {
                        round_data.changes = None;
                    }

                    round_data.customOdds = Some(temp_odds);
                }
            }
        }

        // then, apply custom odds if necessary
        if let Some(custom_odds) = &self.custom_odds {
            let mut temp_odds = round_data.customOdds.unwrap_or(round_data.currentOdds);
            round_data
                .pirates
                .iter()
                .enumerate()
                .for_each(|(arena_index, arena)| {
                    arena.iter().enumerate().for_each(|(pirate_index, pirate)| {
                        if let Some(odds) = custom_odds.get(pirate) {
                            temp_odds[arena_index][pirate_index + 1] = *odds;
                        }
                    });
                });

            round_data.customOdds = Some(temp_odds);
        }
    }

    /// Returns a deep copy of the modifier.
    pub fn copy(&self) -> Self {
        Self {
            value: self.value,
            custom_odds: self.custom_odds.clone(),
            custom_time: self.custom_time,
        }
    }
}
