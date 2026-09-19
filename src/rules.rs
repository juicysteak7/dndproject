//! 5e rules math. Pure functions, shared by server and browser so the sheet
//! recomputes instantly on the client and the server agrees with it.

use serde::{Deserialize, Serialize};

/// The six ability scores, in the canonical sheet order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ability {
    Str,
    Dex,
    Con,
    Int,
    Wis,
    Cha,
}

impl Ability {
    pub const ALL: [Ability; 6] = [
        Ability::Str,
        Ability::Dex,
        Ability::Con,
        Ability::Int,
        Ability::Wis,
        Ability::Cha,
    ];

    /// Stable key used in JSON columns and form fields.
    pub fn key(self) -> &'static str {
        match self {
            Ability::Str => "str",
            Ability::Dex => "dex",
            Ability::Con => "con",
            Ability::Int => "int",
            Ability::Wis => "wis",
            Ability::Cha => "cha",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Ability::Str => "Strength",
            Ability::Dex => "Dexterity",
            Ability::Con => "Constitution",
            Ability::Int => "Intelligence",
            Ability::Wis => "Wisdom",
            Ability::Cha => "Charisma",
        }
    }

    pub fn from_key(key: &str) -> Option<Ability> {
        Ability::ALL.into_iter().find(|a| a.key() == key)
    }
}

/// The 18 skills and the ability each keys off.
pub const SKILLS: [(&str, &str, Ability); 18] = [
    ("acrobatics", "Acrobatics", Ability::Dex),
    ("animal_handling", "Animal Handling", Ability::Wis),
    ("arcana", "Arcana", Ability::Int),
    ("athletics", "Athletics", Ability::Str),
    ("deception", "Deception", Ability::Cha),
    ("history", "History", Ability::Int),
    ("insight", "Insight", Ability::Wis),
    ("intimidation", "Intimidation", Ability::Cha),
    ("investigation", "Investigation", Ability::Int),
    ("medicine", "Medicine", Ability::Wis),
    ("nature", "Nature", Ability::Int),
    ("perception", "Perception", Ability::Wis),
    ("performance", "Performance", Ability::Cha),
    ("persuasion", "Persuasion", Ability::Cha),
    ("religion", "Religion", Ability::Int),
    ("sleight_of_hand", "Sleight of Hand", Ability::Dex),
    ("stealth", "Stealth", Ability::Dex),
    ("survival", "Survival", Ability::Wis),
];

/// Standard 5e conditions, offered in the combat tracker.
pub const CONDITIONS: [&str; 15] = [
    "Blinded",
    "Charmed",
    "Deafened",
    "Exhaustion",
    "Frightened",
    "Grappled",
    "Incapacitated",
    "Invisible",
    "Paralyzed",
    "Petrified",
    "Poisoned",
    "Prone",
    "Restrained",
    "Stunned",
    "Unconscious",
];

/// `floor((score - 10) / 2)`, correct for scores below 10 where naive integer
/// division truncates toward zero instead of down.
pub fn ability_modifier(score: i64) -> i64 {
    (score - 10).div_euclid(2)
}

/// Proficiency bonus by character level: +2 at 1-4, rising every 4 levels.
pub fn proficiency_bonus(level: i64) -> i64 {
    2 + (level.clamp(1, 20) - 1) / 4
}

/// Skill check bonus, accounting for proficiency and expertise (double prof).
pub fn skill_bonus(score: i64, level: i64, proficient: bool, expert: bool) -> i64 {
    let pb = proficiency_bonus(level);
    let mult = if expert {
        2
    } else if proficient {
        1
    } else {
        0
    };
    ability_modifier(score) + pb * mult
}

/// Saving throw bonus.
pub fn save_bonus(score: i64, level: i64, proficient: bool) -> i64 {
    ability_modifier(score) + if proficient { proficiency_bonus(level) } else { 0 }
}

/// Passive score for a skill: 10 + the skill bonus.
pub fn passive_score(skill_bonus: i64) -> i64 {
    10 + skill_bonus
}

/// XP thresholds for levels 1-20 (5e PHB).
pub const XP_THRESHOLDS: [i64; 20] = [
    0, 300, 900, 2700, 6500, 14000, 23000, 34000, 48000, 64000, 85000, 100000, 120000, 140000,
    165000, 195000, 225000, 265000, 305000, 355000,
];

/// The level a given XP total earns.
pub fn level_for_xp(xp: i64) -> i64 {
    let mut level = 1;
    for (i, threshold) in XP_THRESHOLDS.iter().enumerate() {
        if xp >= *threshold {
            level = i as i64 + 1;
        }
    }
    level
}

/// Format a bonus the way a sheet shows it: always signed.
pub fn format_bonus(bonus: i64) -> String {
    if bonus >= 0 {
        format!("+{bonus}")
    } else {
        bonus.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers_round_down_for_low_scores() {
        assert_eq!(ability_modifier(1), -5);
        assert_eq!(ability_modifier(7), -2);
        assert_eq!(ability_modifier(8), -1);
        assert_eq!(ability_modifier(9), -1);
        assert_eq!(ability_modifier(10), 0);
        assert_eq!(ability_modifier(11), 0);
        assert_eq!(ability_modifier(12), 1);
        assert_eq!(ability_modifier(20), 5);
        assert_eq!(ability_modifier(30), 10);
    }

    #[test]
    fn proficiency_steps_every_four_levels() {
        assert_eq!(proficiency_bonus(1), 2);
        assert_eq!(proficiency_bonus(4), 2);
        assert_eq!(proficiency_bonus(5), 3);
        assert_eq!(proficiency_bonus(9), 4);
        assert_eq!(proficiency_bonus(13), 5);
        assert_eq!(proficiency_bonus(17), 6);
        assert_eq!(proficiency_bonus(20), 6);
    }

    #[test]
    fn expertise_doubles_proficiency() {
        // Level 5 rogue, Dex 18: +4 mod, +3 prof.
        assert_eq!(skill_bonus(18, 5, false, false), 4);
        assert_eq!(skill_bonus(18, 5, true, false), 7);
        assert_eq!(skill_bonus(18, 5, true, true), 10);
    }

    #[test]
    fn passive_perception_matches_sheet() {
        // Wis 14 (+2), level 1, proficient (+2) => bonus 4 => passive 14.
        assert_eq!(passive_score(skill_bonus(14, 1, true, false)), 14);
    }

    #[test]
    fn xp_maps_to_level() {
        assert_eq!(level_for_xp(0), 1);
        assert_eq!(level_for_xp(299), 1);
        assert_eq!(level_for_xp(300), 2);
        assert_eq!(level_for_xp(6499), 4);
        assert_eq!(level_for_xp(6500), 5);
        assert_eq!(level_for_xp(355000), 20);
        assert_eq!(level_for_xp(999999), 20);
    }

    #[test]
    fn bonuses_are_always_signed() {
        assert_eq!(format_bonus(3), "+3");
        assert_eq!(format_bonus(0), "+0");
        assert_eq!(format_bonus(-1), "-1");
    }

    #[test]
    fn every_skill_has_a_unique_key() {
        let mut keys: Vec<&str> = SKILLS.iter().map(|(k, _, _)| *k).collect();
        keys.sort_unstable();
        let count = keys.len();
        keys.dedup();
        assert_eq!(keys.len(), count);
    }
}
