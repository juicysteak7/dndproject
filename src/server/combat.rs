use leptos::prelude::*;

use crate::models::{Combatant, Encounter};

#[server(ListEncounters, "/api")]
pub async fn list_encounters() -> Result<Vec<Encounter>, ServerFnError> {
    use crate::db;

    sqlx::query_as::<_, Encounter>("SELECT * FROM encounters ORDER BY created_at DESC")
        .fetch_all(db::pool())
        .await
        .map_err(super::to_server_err)
}

#[server(GetEncounter, "/api")]
pub async fn get_encounter(id: String) -> Result<Option<Encounter>, ServerFnError> {
    use crate::db;

    sqlx::query_as::<_, Encounter>("SELECT * FROM encounters WHERE id = ?")
        .bind(&id)
        .fetch_optional(db::pool())
        .await
        .map_err(super::to_server_err)
}

#[server(CreateEncounter, "/api")]
pub async fn create_encounter(name: String) -> Result<String, ServerFnError> {
    use crate::db;

    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(ServerFnError::new("Encounter needs a name"));
    }

    let party_id = db::ensure_default_party()
        .await
        .map_err(super::to_server_err)?;
    let id = db::new_id();

    sqlx::query(
        "INSERT INTO encounters (id, party_id, name, round, turn_index, status, created_at)
         VALUES (?, ?, ?, 1, 0, 'planning', ?)",
    )
    .bind(&id)
    .bind(&party_id)
    .bind(&name)
    .bind(db::now())
    .execute(db::pool())
    .await
    .map_err(super::to_server_err)?;

    Ok(id)
}

#[server(DeleteEncounter, "/api")]
pub async fn delete_encounter(id: String) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query("DELETE FROM encounters WHERE id = ?")
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
    Ok(())
}

/// Combatants in initiative order.
#[server(ListCombatants, "/api")]
pub async fn list_combatants(encounter_id: String) -> Result<Vec<Combatant>, ServerFnError> {
    use crate::db;

    let mut rows = sqlx::query_as::<_, Combatant>("SELECT * FROM combatants WHERE encounter_id = ?")
        .bind(&encounter_id)
        .fetch_all(db::pool())
        .await
        .map_err(super::to_server_err)?;

    crate::models::sort_initiative(&mut rows);
    Ok(rows)
}

#[server(AddCombatant, "/api")]
pub async fn add_combatant(
    encounter_id: String,
    name: String,
    initiative: i64,
    hp_max: i64,
    armor_class: i64,
    dex_score: i64,
) -> Result<(), ServerFnError> {
    use crate::db;

    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(ServerFnError::new("Combatant needs a name"));
    }
    let hp_max = hp_max.max(1);

    sqlx::query(
        "INSERT INTO combatants
            (id, encounter_id, character_id, name, initiative, dex_score,
             hp_max, hp_current, hp_temp, armor_class, conditions, concentrating, is_pc, notes)
         VALUES (?, ?, NULL, ?, ?, ?, ?, ?, 0, ?, '[]', 0, 0, '')",
    )
    .bind(db::new_id())
    .bind(&encounter_id)
    .bind(&name)
    .bind(initiative)
    .bind(dex_score.clamp(1, 30))
    .bind(hp_max)
    .bind(hp_max)
    .bind(armor_class.max(0))
    .execute(db::pool())
    .await
    .map_err(super::to_server_err)?;

    Ok(())
}

/// Pull every active party member into the encounter, skipping any already in it.
#[server(AddPartyToEncounter, "/api")]
pub async fn add_party_to_encounter(encounter_id: String) -> Result<u64, ServerFnError> {
    use crate::db;

    let characters = super::characters::list_characters().await?;
    let existing: Vec<(String,)> = sqlx::query_as(
        "SELECT character_id FROM combatants WHERE encounter_id = ? AND character_id IS NOT NULL",
    )
    .bind(&encounter_id)
    .fetch_all(db::pool())
    .await
    .map_err(super::to_server_err)?;

    let already: Vec<String> = existing.into_iter().map(|(id,)| id).collect();
    let mut added = 0u64;

    for c in characters {
        if already.contains(&c.id) {
            continue;
        }
        sqlx::query(
            "INSERT INTO combatants
                (id, encounter_id, character_id, name, initiative, dex_score,
                 hp_max, hp_current, hp_temp, armor_class, conditions, concentrating, is_pc, notes)
             VALUES (?, ?, ?, ?, 0, ?, ?, ?, ?, ?, '[]', 0, 1, '')",
        )
        .bind(db::new_id())
        .bind(&encounter_id)
        .bind(&c.id)
        .bind(&c.name)
        .bind(c.dex_score)
        .bind(c.hp_max)
        .bind(c.hp_current)
        .bind(c.hp_temp)
        .bind(c.armor_class)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
        added += 1;
    }

    Ok(added)
}

#[server(RemoveCombatant, "/api")]
pub async fn remove_combatant(id: String) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query("DELETE FROM combatants WHERE id = ?")
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
    Ok(())
}

#[server(SetInitiative, "/api")]
pub async fn set_initiative(id: String, initiative: i64) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query("UPDATE combatants SET initiative = ? WHERE id = ?")
        .bind(initiative)
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
    Ok(())
}

/// Roll d20 + Dex modifier for every combatant that has not yet acted.
#[server(RollAllInitiative, "/api")]
pub async fn roll_all_initiative(encounter_id: String) -> Result<(), ServerFnError> {
    use crate::db;
    use crate::rules::ability_modifier;

    let combatants = list_combatants(encounter_id.clone()).await?;
    let d20 = crate::dice::parse("d20").map_err(super::to_server_err)?;

    for c in combatants {
        let roll = crate::dice::roll(d20);
        let initiative = roll.total + ability_modifier(c.dex_score);

        sqlx::query("UPDATE combatants SET initiative = ? WHERE id = ?")
            .bind(initiative)
            .bind(&c.id)
            .execute(db::pool())
            .await
            .map_err(super::to_server_err)?;
    }

    // A fresh roll restarts the order.
    sqlx::query("UPDATE encounters SET turn_index = 0, round = 1, status = 'running' WHERE id = ?")
        .bind(&encounter_id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;

    Ok(())
}

#[server(AdjustCombatantHp, "/api")]
pub async fn adjust_combatant_hp(id: String, delta: i64) -> Result<(), ServerFnError> {
    use crate::db;

    let c = sqlx::query_as::<_, Combatant>("SELECT * FROM combatants WHERE id = ?")
        .bind(&id)
        .fetch_optional(db::pool())
        .await
        .map_err(super::to_server_err)?
        .ok_or_else(|| ServerFnError::new("No such combatant"))?;

    let (hp_current, hp_temp) =
        super::characters::apply_hp_delta(c.hp_current, c.hp_temp, c.hp_max, delta);

    sqlx::query("UPDATE combatants SET hp_current = ?, hp_temp = ? WHERE id = ?")
        .bind(hp_current)
        .bind(hp_temp)
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;

    // Keep the linked character sheet in sync so HP survives the encounter.
    if let Some(character_id) = c.character_id {
        sqlx::query("UPDATE characters SET hp_current = ?, hp_temp = ?, updated_at = ? WHERE id = ?")
            .bind(hp_current)
            .bind(hp_temp)
            .bind(db::now())
            .bind(&character_id)
            .execute(db::pool())
            .await
            .map_err(super::to_server_err)?;
    }

    Ok(())
}

#[server(ToggleCondition, "/api")]
pub async fn toggle_condition(id: String, condition: String) -> Result<(), ServerFnError> {
    use crate::db;

    let c = sqlx::query_as::<_, Combatant>("SELECT * FROM combatants WHERE id = ?")
        .bind(&id)
        .fetch_optional(db::pool())
        .await
        .map_err(super::to_server_err)?
        .ok_or_else(|| ServerFnError::new("No such combatant"))?;

    let mut conditions = c.condition_list();
    match conditions.iter().position(|x| *x == condition) {
        Some(idx) => {
            conditions.remove(idx);
        }
        None => conditions.push(condition),
    }

    sqlx::query("UPDATE combatants SET conditions = ? WHERE id = ?")
        .bind(serde_json::to_string(&conditions).unwrap_or_else(|_| "[]".into()))
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;

    Ok(())
}

#[server(ToggleConcentration, "/api")]
pub async fn toggle_concentration(id: String) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query("UPDATE combatants SET concentrating = 1 - concentrating WHERE id = ?")
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
    Ok(())
}

/// Advance the turn pointer, wrapping to the next round at the end of the order.
///
/// Pure so the wrap-around is unit tested.
pub fn advance_turn(turn_index: i64, round: i64, combatant_count: i64) -> (i64, i64) {
    if combatant_count <= 0 {
        return (0, round);
    }
    let next = turn_index + 1;
    if next >= combatant_count {
        (0, round + 1)
    } else {
        (next, round)
    }
}

/// Step back a turn, unwinding into the previous round if needed.
pub fn rewind_turn(turn_index: i64, round: i64, combatant_count: i64) -> (i64, i64) {
    if combatant_count <= 0 {
        return (0, round);
    }
    if turn_index == 0 {
        // Never go back past the start of the fight.
        if round <= 1 {
            (0, 1)
        } else {
            (combatant_count - 1, round - 1)
        }
    } else {
        (turn_index - 1, round)
    }
}

#[server(NextTurn, "/api")]
pub async fn next_turn(encounter_id: String) -> Result<(), ServerFnError> {
    use crate::db;

    let encounter = get_encounter(encounter_id.clone())
        .await?
        .ok_or_else(|| ServerFnError::new("No such encounter"))?;
    let count = list_combatants(encounter_id.clone()).await?.len() as i64;

    let (turn_index, round) = advance_turn(encounter.turn_index, encounter.round, count);

    sqlx::query("UPDATE encounters SET turn_index = ?, round = ?, status = 'running' WHERE id = ?")
        .bind(turn_index)
        .bind(round)
        .bind(&encounter_id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;

    Ok(())
}

#[server(PreviousTurn, "/api")]
pub async fn previous_turn(encounter_id: String) -> Result<(), ServerFnError> {
    use crate::db;

    let encounter = get_encounter(encounter_id.clone())
        .await?
        .ok_or_else(|| ServerFnError::new("No such encounter"))?;
    let count = list_combatants(encounter_id.clone()).await?.len() as i64;

    let (turn_index, round) = rewind_turn(encounter.turn_index, encounter.round, count);

    sqlx::query("UPDATE encounters SET turn_index = ?, round = ? WHERE id = ?")
        .bind(turn_index)
        .bind(round)
        .bind(&encounter_id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;

    Ok(())
}

#[server(EndEncounter, "/api")]
pub async fn end_encounter(encounter_id: String) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query("UPDATE encounters SET status = 'done' WHERE id = ?")
        .bind(&encounter_id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns_cycle_and_increment_the_round() {
        // 3 combatants: 0 -> 1 -> 2 -> back to 0 on round 2.
        assert_eq!(advance_turn(0, 1, 3), (1, 1));
        assert_eq!(advance_turn(1, 1, 3), (2, 1));
        assert_eq!(advance_turn(2, 1, 3), (0, 2));
    }

    #[test]
    fn advancing_an_empty_encounter_is_a_no_op() {
        assert_eq!(advance_turn(0, 1, 0), (0, 1));
    }

    #[test]
    fn rewinding_unwinds_into_the_previous_round() {
        assert_eq!(rewind_turn(1, 2, 3), (0, 2));
        assert_eq!(rewind_turn(0, 2, 3), (2, 1));
    }

    #[test]
    fn rewinding_stops_at_the_start_of_the_fight() {
        assert_eq!(rewind_turn(0, 1, 3), (0, 1));
    }

    #[test]
    fn a_single_combatant_advances_the_round_every_turn() {
        assert_eq!(advance_turn(0, 1, 1), (0, 2));
        assert_eq!(advance_turn(0, 2, 1), (0, 3));
    }
}
