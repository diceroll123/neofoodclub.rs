use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use crate::{
    arena::ARENA_NAMES,
    nfc::NeoFoodClub,
    pirates::PartialPirate,
    utils::{convert_from_utc_to_nst, timestamp_to_utc},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OddsChange {
    pub t: String,
    pub new: u8,
    pub old: u8,
    arena: u8,
    pirate: u8,
}

impl OddsChange {
    pub fn pirate(&self, nfc: &NeoFoodClub) -> PartialPirate {
        PartialPirate {
            id: self.pirate_id(nfc),
        }
    }

    pub fn pirate_id(&self, nfc: &NeoFoodClub) -> usize {
        nfc.pirates()[self.arena_index()][self.pirate_index() - 1] as usize
    }

    pub fn arena(&self) -> &str {
        ARENA_NAMES[self.arena as usize]
    }

    #[inline]
    pub fn pirate_index(&self) -> usize {
        self.pirate as usize
    }

    #[inline]
    pub fn arena_index(&self) -> usize {
        self.arena as usize
    }

    pub fn timestamp_nst(&self) -> DateTime<Tz> {
        convert_from_utc_to_nst(self.timestamp_utc())
    }

    pub fn timestamp_utc(&self) -> DateTime<Utc> {
        timestamp_to_utc(&self.t)
    }
}
