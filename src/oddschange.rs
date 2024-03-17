use serde::{Deserialize, Serialize};

use crate::{arena::ARENA_NAMES, nfc::NeoFoodClub, pirates::PartialPirate};

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
            id: nfc.pirates()[self.arena as usize][self.pirate as usize - 1] as usize,
        }
    }

    pub fn arena(&self) -> &str {
        ARENA_NAMES[self.arena as usize]
    }

    pub fn pirate_index(&self) -> usize {
        self.pirate as usize - 1
    }

    pub fn arena_index(&self) -> usize {
        self.arena as usize
    }
}
