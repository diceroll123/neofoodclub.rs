use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use crate::{
    arena::ARENA_NAMES,
    nfc::NeoFoodClub,
    pirates::PartialPirate,
    utils::{convert_from_utc_to_nst, timestamp_to_utc},
};

/// Represents a change in odds.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OddsChange {
    pub t: String,
    pub new: u8,
    pub old: u8,
    arena: u8,
    pirate: u8,
}

impl OddsChange {
    /// Returns the pirate associated with the change.
    pub fn pirate(&self, nfc: &NeoFoodClub) -> PartialPirate {
        PartialPirate {
            id: self.pirate_id(nfc),
        }
    }

    /// Returns the pirate ID associated with the change.
    pub fn pirate_id(&self, nfc: &NeoFoodClub) -> usize {
        nfc.pirates()[self.arena_index()][self.pirate_index() - 1] as usize
    }

    /// Returns the name of the arena associated with the change.
    pub fn arena(&self) -> &str {
        ARENA_NAMES[self.arena as usize]
    }

    /// Returns the index of the pirate associated with the change.
    #[inline]
    pub fn pirate_index(&self) -> usize {
        self.pirate as usize
    }

    /// Returns the index of the arena associated with the change.
    #[inline]
    pub fn arena_index(&self) -> usize {
        self.arena as usize
    }

    /// Returns the timestamp of the change in NST.
    pub fn timestamp_nst(&self) -> DateTime<Tz> {
        convert_from_utc_to_nst(self.timestamp_utc())
    }

    /// Returns the timestamp of the change in UTC.
    pub fn timestamp_utc(&self) -> DateTime<Utc> {
        timestamp_to_utc(&self.t)
    }
}
