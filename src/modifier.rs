use bitflags::bitflags;

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

#[derive(Debug, Clone, Default)]
pub struct Modifier {
    pub value: i32,
}

impl Modifier {
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

impl Modifier {
    // flags

    fn has_flag(&self, flag: i32) -> bool {
        self.value & flag == flag
    }

    pub fn is_empty(&self) -> bool {
        self.has_flag(ModifierFlags::EMPTY.bits())
    }

    pub fn is_general(&self) -> bool {
        self.has_flag(ModifierFlags::GENERAL.bits())
    }

    pub fn is_opening_odds(&self) -> bool {
        self.has_flag(ModifierFlags::OPENING_ODDS.bits())
    }

    pub fn is_reverse(&self) -> bool {
        self.has_flag(ModifierFlags::REVERSE.bits())
    }

    pub fn is_charity_corner(&self) -> bool {
        self.has_flag(ModifierFlags::CHARITY_CORNER.bits())
    }
}
