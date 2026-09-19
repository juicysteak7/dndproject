//! Dice notation parsing and rolling.
//!
//! The parser is shared (the browser validates input as you type); the actual
//! rolling happens server-side so the wasm build needs no RNG entropy source.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiceExpr {
    pub count: u32,
    pub sides: u32,
    pub modifier: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DiceError {
    #[error("empty dice expression")]
    Empty,
    #[error("expected NdM notation, e.g. 2d6+3")]
    Malformed,
    #[error("dice count must be between 1 and 100")]
    BadCount,
    #[error("die must have between 2 and 1000 sides")]
    BadSides,
}

impl fmt::Display for DiceExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}d{}", self.count, self.sides)?;
        match self.modifier {
            0 => Ok(()),
            m if m > 0 => write!(f, "+{m}"),
            m => write!(f, "{m}"),
        }
    }
}

/// Parse dice notation: `d20`, `2d6`, `4d6+3`, `1d8 - 1`.
pub fn parse(input: &str) -> Result<DiceExpr, DiceError> {
    let cleaned: String = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();

    if cleaned.is_empty() {
        return Err(DiceError::Empty);
    }

    let (dice_part, modifier) = match cleaned.find(['+', '-']) {
        Some(idx) => {
            let (d, m) = cleaned.split_at(idx);
            let modifier: i64 = m.parse().map_err(|_| DiceError::Malformed)?;
            (d, modifier)
        }
        None => (cleaned.as_str(), 0),
    };

    let (count_str, sides_str) = dice_part.split_once('d').ok_or(DiceError::Malformed)?;

    // A bare `d20` means one d20.
    let count: u32 = if count_str.is_empty() {
        1
    } else {
        count_str.parse().map_err(|_| DiceError::Malformed)?
    };
    let sides: u32 = sides_str.parse().map_err(|_| DiceError::Malformed)?;

    if !(1..=100).contains(&count) {
        return Err(DiceError::BadCount);
    }
    if !(2..=1000).contains(&sides) {
        return Err(DiceError::BadSides);
    }

    Ok(DiceExpr {
        count,
        sides,
        modifier,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollOutcome {
    pub expression: String,
    pub rolls: Vec<u32>,
    pub modifier: i64,
    pub total: i64,
    /// Set for a single d20: whether it was a natural 20 or natural 1.
    pub critical: Option<bool>,
}

/// Roll each die using `next`, a source of values in `1..=sides`.
///
/// Split out from the RNG so it can be tested deterministically.
pub fn roll_with<F>(expr: DiceExpr, mut next: F) -> RollOutcome
where
    F: FnMut(u32) -> u32,
{
    let rolls: Vec<u32> = (0..expr.count).map(|_| next(expr.sides)).collect();
    let sum: i64 = rolls.iter().map(|r| *r as i64).sum();

    let critical = if expr.count == 1 && expr.sides == 20 {
        match rolls[0] {
            20 => Some(true),
            1 => Some(false),
            _ => None,
        }
    } else {
        None
    };

    RollOutcome {
        expression: expr.to_string(),
        rolls,
        modifier: expr.modifier,
        total: sum + expr.modifier,
        critical,
    }
}

#[cfg(feature = "ssr")]
pub fn roll(expr: DiceExpr) -> RollOutcome {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    roll_with(expr, |sides| rng.gen_range(1..=sides))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bare_die() {
        assert_eq!(
            parse("d20").unwrap(),
            DiceExpr {
                count: 1,
                sides: 20,
                modifier: 0
            }
        );
    }

    #[test]
    fn parses_count_and_modifier() {
        assert_eq!(
            parse("2d6+3").unwrap(),
            DiceExpr {
                count: 2,
                sides: 6,
                modifier: 3
            }
        );
        assert_eq!(
            parse("1d8-1").unwrap(),
            DiceExpr {
                count: 1,
                sides: 8,
                modifier: -1
            }
        );
    }

    #[test]
    fn tolerates_whitespace_and_case() {
        assert_eq!(parse(" 4D6 + 2 ").unwrap().count, 4);
        assert_eq!(parse(" 4D6 + 2 ").unwrap().sides, 6);
        assert_eq!(parse(" 4D6 + 2 ").unwrap().modifier, 2);
    }

    #[test]
    fn rejects_nonsense() {
        assert_eq!(parse(""), Err(DiceError::Empty));
        assert_eq!(parse("hello"), Err(DiceError::Malformed));
        assert_eq!(parse("0d6"), Err(DiceError::BadCount));
        assert_eq!(parse("101d6"), Err(DiceError::BadCount));
        assert_eq!(parse("1d1"), Err(DiceError::BadSides));
    }

    #[test]
    fn round_trips_through_display() {
        for s in ["2d6+3", "1d20", "4d8-2"] {
            assert_eq!(parse(s).unwrap().to_string(), s);
        }
    }

    #[test]
    fn totals_include_modifier() {
        let expr = parse("3d6+2").unwrap();
        let outcome = roll_with(expr, |_| 4);
        assert_eq!(outcome.rolls, vec![4, 4, 4]);
        assert_eq!(outcome.total, 14);
    }

    #[test]
    fn flags_natural_twenty_and_one() {
        let d20 = parse("d20").unwrap();
        assert_eq!(roll_with(d20, |_| 20).critical, Some(true));
        assert_eq!(roll_with(d20, |_| 1).critical, Some(false));
        assert_eq!(roll_with(d20, |_| 11).critical, None);
        // Multiple dice are never flagged.
        assert_eq!(roll_with(parse("2d20").unwrap(), |_| 20).critical, None);
    }
}
