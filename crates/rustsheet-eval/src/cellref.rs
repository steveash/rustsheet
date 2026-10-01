//! A1-style cell address helpers used by the harness (independent of the engine).

use std::fmt;

use serde::{Deserialize, Serialize};

/// 1-based (row, column) address within a sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Addr {
    pub row: u32,
    pub col: u32,
}

impl Addr {
    /// Parses `A1`, `$A$1`, `xfd1048576` (case-insensitive). Returns `None` on malformed input.
    #[must_use]
    pub fn parse(s: &str) -> Option<Addr> {
        let s = s.trim().replace('$', "");
        let split = s.find(|c: char| c.is_ascii_digit())?;
        let (letters, digits) = s.split_at(split);
        if letters.is_empty()
            || letters.len() > 3
            || !letters.chars().all(|c| c.is_ascii_alphabetic())
        {
            return None;
        }
        let mut col: u32 = 0;
        for c in letters.chars() {
            col = col * 26 + (u32::from(c.to_ascii_uppercase() as u8 - b'A') + 1);
        }
        let row: u32 = digits.parse().ok()?;
        if row == 0 || row > 1_048_576 || col > 16_384 {
            return None;
        }
        Some(Addr { row, col })
    }

    /// Column number to letters (1 → `A`, 27 → `AA`).
    #[must_use]
    pub fn col_letters(mut col: u32) -> String {
        let mut out = Vec::new();
        while col > 0 {
            let rem = (col - 1) % 26;
            out.push(char::from(b'A' + u8::try_from(rem).unwrap_or(0)));
            col = (col - 1) / 26;
        }
        out.iter().rev().collect()
    }
}

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", Addr::col_letters(self.col), self.row)
    }
}

/// Parses a range like `A1:C3` (or a single cell) into its inclusive corners.
#[must_use]
pub fn parse_range(s: &str) -> Option<(Addr, Addr)> {
    if let Some((a, b)) = s.split_once(':') {
        Some((Addr::parse(a)?, Addr::parse(b)?))
    } else {
        let a = Addr::parse(s)?;
        Some((a, a))
    }
}

/// Serde adapter: `BTreeMap<Addr, T>` as a JSON object keyed by A1 strings.
pub mod a1_map {
    use std::collections::BTreeMap;

    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::Addr;

    /// # Errors
    /// Propagates serializer errors.
    pub fn serialize<S: Serializer, T: Serialize>(
        map: &BTreeMap<Addr, T>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        s.collect_map(map.iter().map(|(k, v)| (k.to_string(), v)))
    }

    /// # Errors
    /// Fails on keys that are not valid A1 addresses.
    pub fn deserialize<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
        d: D,
    ) -> Result<BTreeMap<Addr, T>, D::Error> {
        let raw: BTreeMap<String, T> = BTreeMap::deserialize(d)?;
        raw.into_iter()
            .map(|(k, v)| {
                Addr::parse(&k)
                    .map(|a| (a, v))
                    .ok_or_else(|| D::Error::custom(format!("bad cell address {k}")))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        for s in ["A1", "Z9", "AA10", "XFD1048576"] {
            let a = Addr::parse(s);
            assert_eq!(a.map(|a| a.to_string()).as_deref(), Some(s));
        }
        assert_eq!(Addr::parse("$b$2"), Some(Addr { row: 2, col: 2 }));
        assert_eq!(Addr::parse("A0"), None);
        assert_eq!(Addr::parse("1A"), None);
        assert_eq!(Addr::parse("XFE1"), None);
    }

    #[test]
    fn ranges() {
        assert_eq!(
            parse_range("A1:B2"),
            Some((Addr { row: 1, col: 1 }, Addr { row: 2, col: 2 }))
        );
    }
}
