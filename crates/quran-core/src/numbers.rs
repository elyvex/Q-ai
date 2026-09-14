//! Validated numeric newtypes for Quran addressing.
//!
//! # quran_core::numbers
//!
//! Construction is the only way in, so no later code can hold an out-of-range
//! surah/ayah/position. These types are `Copy`, ordered, and serde-transparent
//! as plain integers.

use std::fmt;
use std::str::FromStr;

use crate::error::QuranError;

/// A surah number in `1..=114`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurahNumber(u16);

impl SurahNumber {
    /// The smallest valid surah number.
    pub const MIN: u16 = 1;
    /// The largest valid surah number.
    pub const MAX: u16 = 114;

    /// Validate and construct a surah number.
    pub fn new(number: u16) -> Result<Self, QuranError> {
        if (Self::MIN..=Self::MAX).contains(&number) {
            Ok(Self(number))
        } else {
            Err(QuranError::InvalidSurahNumber { number })
        }
    }

    /// The underlying value.
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl fmt::Display for SurahNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SurahNumber {
    type Err = QuranError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let number: u16 = s.parse().map_err(|_| QuranError::InvalidSurahNumber { number: 0 })?;
        Self::new(number)
    }
}

/// An ayah number, 1-based and unbounded above (edition-specific counts).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AyahNumber(u32);

impl AyahNumber {
    /// Validate and construct an ayah number.
    pub fn new(number: u32) -> Result<Self, QuranError> {
        if number >= 1 { Ok(Self(number)) } else { Err(QuranError::InvalidAyahNumber) }
    }

    /// The underlying value.
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for AyahNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for AyahNumber {
    type Err = QuranError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let number: u32 = s.parse().map_err(|_| QuranError::InvalidAyahNumber)?;
        Self::new(number)
    }
}

/// A token position within an ayah, 1-based.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenPosition(u16);

impl TokenPosition {
    /// Validate and construct a token position.
    pub fn new(position: u16) -> Result<Self, QuranError> {
        if position >= 1 { Ok(Self(position)) } else { Err(QuranError::InvalidTokenPosition) }
    }

    /// The underlying value.
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl fmt::Display for TokenPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TokenPosition {
    type Err = QuranError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let position: u16 = s.parse().map_err(|_| QuranError::InvalidTokenPosition)?;
        Self::new(position)
    }
}

macro_rules! serde_integer_newtype {
    ($ty:ty, $inner:ty) => {
        impl serde::Serialize for $ty {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_u64(self.get() as u64)
            }
        }

        impl<'de> serde::Deserialize<'de> for $ty {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let raw = <$inner>::deserialize(deserializer)?;
                Self::new(raw).map_err(serde::de::Error::custom)
            }
        }
    };
}

serde_integer_newtype!(SurahNumber, u16);
serde_integer_newtype!(AyahNumber, u32);
serde_integer_newtype!(TokenPosition, u16);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surah_bounds() {
        assert_eq!(SurahNumber::new(1).unwrap().get(), 1);
        assert_eq!(SurahNumber::new(114).unwrap().get(), 114);
        assert!(SurahNumber::new(0).is_err());
        assert!(SurahNumber::new(115).is_err());
    }

    #[test]
    fn ayah_and_position_are_one_based() {
        assert_eq!(AyahNumber::new(255).unwrap().get(), 255);
        assert!(AyahNumber::new(0).is_err());
        assert_eq!(TokenPosition::new(3).unwrap().get(), 3);
        assert!(TokenPosition::new(0).is_err());
    }

    #[test]
    fn parsing_and_display_roundtrip() {
        assert_eq!("36".parse::<SurahNumber>().unwrap().get(), 36);
        assert!("0".parse::<SurahNumber>().is_err());
        assert!("x".parse::<AyahNumber>().is_err());
        assert_eq!(TokenPosition::new(2).unwrap().to_string(), "2");
    }

    #[test]
    fn serde_is_integer_shaped_and_validates() {
        let s = serde_json::to_string(&SurahNumber::new(2).unwrap()).unwrap();
        assert_eq!(s, "2");
        assert!(serde_json::from_str::<SurahNumber>("0").is_err());
        assert_eq!(serde_json::from_str::<AyahNumber>("255").unwrap().get(), 255);
    }
}
