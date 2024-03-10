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
