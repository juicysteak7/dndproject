//! Domain types shared by the server and the browser.
//!
//! These mirror the tables in `migrations/`. SQLite hands back every integer as
//! `i64`, so the models use `i64` throughout rather than fighting the driver.

use serde::{Deserialize, Serialize};

use crate::rules::{self, Ability};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Party {
    pub id: String,
    pub owner_id: Option<String>,
    pub name: String,
    pub notes: String,
    pub gold: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Character {
    pub id: String,
    pub party_id: Option<String>,
    pub owner_id: Option<String>,
    pub name: String,
    pub player_name: String,
    pub class: String,
    pub subclass: String,
    pub race: String,
    pub background: String,
    pub alignment: String,
    pub level: i64,
    pub xp: i64,

    pub str_score: i64,
    pub dex_score: i64,
    pub con_score: i64,
    pub int_score: i64,
    pub wis_score: i64,
    pub cha_score: i64,

    pub hp_max: i64,
    pub hp_current: i64,
    pub hp_temp: i64,
    pub armor_class: i64,
    pub speed: i64,
    pub hit_dice: String,

    pub death_save_successes: i64,
    pub death_save_failures: i64,
    pub inspiration: i64,

    pub skill_proficiencies: String,
    pub skill_expertise: String,
    pub save_proficiencies: String,

    pub notes: String,
    pub is_active: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl Character {
    pub fn ability_score(&self, ability: Ability) -> i64 {
        match ability {
            Ability::Str => self.str_score,
            Ability::Dex => self.dex_score,
            Ability::Con => self.con_score,
            Ability::Int => self.int_score,
            Ability::Wis => self.wis_score,
            Ability::Cha => self.cha_score,
        }
    }

    pub fn ability_modifier(&self, ability: Ability) -> i64 {
        rules::ability_modifier(self.ability_score(ability))
    }

    pub fn proficiency_bonus(&self) -> i64 {
        rules::proficiency_bonus(self.level)
    }

    /// Decode a JSON array column, tolerating corruption by falling back to
    /// empty rather than failing the whole page render.
    fn decode_keys(raw: &str) -> Vec<String> {
        serde_json::from_str(raw).unwrap_or_default()
    }

    pub fn skill_proficiency_keys(&self) -> Vec<String> {
        Self::decode_keys(&self.skill_proficiencies)
    }

    pub fn skill_expertise_keys(&self) -> Vec<String> {
        Self::decode_keys(&self.skill_expertise)
    }

    pub fn save_proficiency_keys(&self) -> Vec<String> {
        Self::decode_keys(&self.save_proficiencies)
    }

    pub fn is_skill_proficient(&self, skill_key: &str) -> bool {
        self.skill_proficiency_keys().iter().any(|k| k == skill_key)
    }

    pub fn has_expertise(&self, skill_key: &str) -> bool {
        self.skill_expertise_keys().iter().any(|k| k == skill_key)
    }

    pub fn is_save_proficient(&self, ability: Ability) -> bool {
        self.save_proficiency_keys()
            .iter()
            .any(|k| k == ability.key())
    }

    pub fn skill_bonus(&self, skill_key: &str) -> Option<i64> {
        let (_, _, ability) = rules::SKILLS.iter().find(|(k, _, _)| *k == skill_key)?;
        Some(rules::skill_bonus(
            self.ability_score(*ability),
            self.level,
            self.is_skill_proficient(skill_key),
            self.has_expertise(skill_key),
        ))
    }

    pub fn save_bonus(&self, ability: Ability) -> i64 {
        rules::save_bonus(
            self.ability_score(ability),
            self.level,
            self.is_save_proficient(ability),
        )
    }

    pub fn initiative_bonus(&self) -> i64 {
        self.ability_modifier(Ability::Dex)
    }

    pub fn passive_perception(&self) -> i64 {
        rules::passive_score(self.skill_bonus("perception").unwrap_or(0))
    }

    /// Effective HP including temporary hit points.
    pub fn effective_hp(&self) -> i64 {
        self.hp_current + self.hp_temp
    }

    pub fn hp_fraction(&self) -> f64 {
        if self.hp_max <= 0 {
            return 0.0;
        }
        (self.hp_current as f64 / self.hp_max as f64).clamp(0.0, 1.0)
    }

    pub fn is_unconscious(&self) -> bool {
        self.hp_current <= 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Item {
    pub id: String,
    pub character_id: String,
    pub name: String,
    pub quantity: i64,
    pub weight: f64,
    pub description: String,
    pub equipped: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Location {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub parent_id: Option<String>,
    pub description: String,
    pub discovered: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Faction {
    pub id: String,
    pub name: String,
    pub description: String,
    pub attitude: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Npc {
    pub id: String,
    pub name: String,
    pub race: String,
    pub role: String,
    pub location_id: Option<String>,
    pub faction_id: Option<String>,
    pub attitude: i64,
    pub armor_class: i64,
    pub hp_max: i64,
    pub challenge_rating: String,
    pub description: String,
    pub secrets: String,
    pub alive: i64,
}

/// How an NPC or faction feels about the party, bucketed for display.
pub fn attitude_label(attitude: i64) -> &'static str {
    match attitude {
        a if a >= 60 => "Devoted",
        a if a >= 25 => "Friendly",
        a if a > -25 => "Neutral",
        a if a > -60 => "Unfriendly",
        _ => "Hostile",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Quest {
    pub id: String,
    pub party_id: Option<String>,
    pub title: String,
    pub status: String,
    pub description: String,
    pub reward: String,
    pub giver_npc_id: Option<String>,
    pub location_id: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

pub const QUEST_STATUSES: [&str; 4] = ["active", "completed", "failed", "rumor"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct GameSession {
    pub id: String,
    pub party_id: Option<String>,
    pub session_number: i64,
    pub played_on: String,
    pub title: String,
    pub summary: String,
    pub notes: String,
    pub xp_awarded: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Encounter {
    pub id: String,
    pub party_id: Option<String>,
    pub name: String,
    pub round: i64,
    pub turn_index: i64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Combatant {
    pub id: String,
    pub encounter_id: String,
    pub character_id: Option<String>,
    pub name: String,
    pub initiative: i64,
    pub dex_score: i64,
    pub hp_max: i64,
    pub hp_current: i64,
    pub hp_temp: i64,
    pub armor_class: i64,
    pub conditions: String,
    pub concentrating: i64,
    pub is_pc: i64,
    pub notes: String,
}

impl Combatant {
    pub fn condition_list(&self) -> Vec<String> {
        serde_json::from_str(&self.conditions).unwrap_or_default()
    }

    pub fn has_condition(&self, condition: &str) -> bool {
        self.condition_list().iter().any(|c| c == condition)
    }

    pub fn hp_fraction(&self) -> f64 {
        if self.hp_max <= 0 {
            return 0.0;
        }
        (self.hp_current as f64 / self.hp_max as f64).clamp(0.0, 1.0)
    }

    pub fn is_down(&self) -> bool {
        self.hp_current <= 0
    }
}

/// Initiative order: highest initiative first, ties broken by Dexterity, then
/// by name so the order is stable across reloads.
pub fn sort_initiative(combatants: &mut [Combatant]) {
    combatants.sort_by(|a, b| {
        b.initiative
            .cmp(&a.initiative)
            .then(b.dex_score.cmp(&a.dex_score))
            .then(a.name.cmp(&b.name))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn combatant(name: &str, initiative: i64, dex: i64) -> Combatant {
        Combatant {
            id: name.to_string(),
            encounter_id: "e".into(),
            character_id: None,
            name: name.to_string(),
            initiative,
            dex_score: dex,
            hp_max: 10,
            hp_current: 10,
            hp_temp: 0,
            armor_class: 10,
            conditions: "[]".into(),
            concentrating: 0,
            is_pc: 0,
            notes: String::new(),
        }
    }

    #[test]
    fn initiative_sorts_high_to_low_with_dex_tiebreak() {
        let mut c = vec![
            combatant("Goblin", 12, 14),
            combatant("Rogue", 19, 18),
            combatant("Bear", 12, 16),
        ];
        sort_initiative(&mut c);
        let order: Vec<&str> = c.iter().map(|x| x.name.as_str()).collect();
        assert_eq!(order, vec!["Rogue", "Bear", "Goblin"]);
    }

    #[test]
    fn identical_initiative_and_dex_sorts_by_name() {
        let mut c = vec![combatant("Zara", 10, 10), combatant("Alix", 10, 10)];
        sort_initiative(&mut c);
        assert_eq!(c[0].name, "Alix");
    }

    #[test]
    fn attitude_buckets() {
        assert_eq!(attitude_label(100), "Devoted");
        assert_eq!(attitude_label(30), "Friendly");
        assert_eq!(attitude_label(0), "Neutral");
        assert_eq!(attitude_label(-30), "Unfriendly");
        assert_eq!(attitude_label(-80), "Hostile");
    }

    #[test]
    fn corrupt_json_columns_degrade_to_empty() {
        let mut c = combatant("X", 0, 10);
        c.conditions = "not json".into();
        assert!(c.condition_list().is_empty());
    }
}
