use leptos::prelude::*;

use crate::models::{Character, Item};

#[server(ListCharacters, "/api")]
pub async fn list_characters() -> Result<Vec<Character>, ServerFnError> {
    use crate::db;

    sqlx::query_as::<_, Character>(
        "SELECT * FROM characters WHERE is_active = 1 ORDER BY name COLLATE NOCASE",
    )
    .fetch_all(db::pool())
    .await
    .map_err(super::to_server_err)
}

#[server(GetCharacter, "/api")]
pub async fn get_character(id: String) -> Result<Option<Character>, ServerFnError> {
    use crate::db;

    sqlx::query_as::<_, Character>("SELECT * FROM characters WHERE id = ?")
        .bind(&id)
        .fetch_optional(db::pool())
        .await
        .map_err(super::to_server_err)
}

#[server(CreateCharacter, "/api")]
pub async fn create_character(
    name: String,
    class: String,
    race: String,
    level: i64,
) -> Result<String, ServerFnError> {
    use crate::db;

    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(ServerFnError::new("Character needs a name"));
    }
    let level = level.clamp(1, 20);
    let party_id = db::ensure_default_party()
        .await
        .map_err(super::to_server_err)?;

    let id = db::new_id();
    let now = db::now();

    sqlx::query(
        "INSERT INTO characters (id, party_id, owner_id, name, class, race, level, created_at, updated_at)
         VALUES (?, ?, NULL, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&party_id)
    .bind(&name)
    .bind(class.trim())
    .bind(race.trim())
    .bind(level)
    .bind(&now)
    .bind(&now)
    .execute(db::pool())
    .await
    .map_err(super::to_server_err)?;

    Ok(id)
}

/// Save the editable fields of a character sheet.
#[server(UpdateCharacter, "/api")]
#[allow(clippy::too_many_arguments)]
pub async fn update_character(
    id: String,
    name: String,
    player_name: String,
    class: String,
    subclass: String,
    race: String,
    background: String,
    alignment: String,
    level: i64,
    xp: i64,
    str_score: i64,
    dex_score: i64,
    con_score: i64,
    int_score: i64,
    wis_score: i64,
    cha_score: i64,
    hp_max: i64,
    hp_current: i64,
    hp_temp: i64,
    armor_class: i64,
    speed: i64,
    hit_dice: String,
    notes: String,
) -> Result<(), ServerFnError> {
    use crate::db;

    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(ServerFnError::new("Character needs a name"));
    }

    let clamp_score = |s: i64| s.clamp(1, 30);
    let hp_max = hp_max.max(1);

    sqlx::query(
        "UPDATE characters SET
            name = ?, player_name = ?, class = ?, subclass = ?, race = ?,
            background = ?, alignment = ?, level = ?, xp = ?,
            str_score = ?, dex_score = ?, con_score = ?,
            int_score = ?, wis_score = ?, cha_score = ?,
            hp_max = ?, hp_current = ?, hp_temp = ?,
            armor_class = ?, speed = ?, hit_dice = ?, notes = ?,
            updated_at = ?
         WHERE id = ?",
    )
    .bind(&name)
    .bind(player_name.trim())
    .bind(class.trim())
    .bind(subclass.trim())
    .bind(race.trim())
    .bind(background.trim())
    .bind(alignment.trim())
    .bind(level.clamp(1, 20))
    .bind(xp.max(0))
    .bind(clamp_score(str_score))
    .bind(clamp_score(dex_score))
    .bind(clamp_score(con_score))
    .bind(clamp_score(int_score))
    .bind(clamp_score(wis_score))
    .bind(clamp_score(cha_score))
    .bind(hp_max)
    .bind(hp_current.clamp(0, hp_max))
    .bind(hp_temp.max(0))
    .bind(armor_class.max(0))
    .bind(speed.max(0))
    .bind(hit_dice.trim())
    .bind(notes)
    .bind(db::now())
    .bind(&id)
    .execute(db::pool())
    .await
    .map_err(super::to_server_err)?;

    Ok(())
}

#[server(DeleteCharacter, "/api")]
pub async fn delete_character(id: String) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query("DELETE FROM characters WHERE id = ?")
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
    Ok(())
}

/// Apply damage (negative) or healing (positive).
#[server(AdjustHp, "/api")]
pub async fn adjust_hp(id: String, delta: i64) -> Result<(), ServerFnError> {
    use crate::db;

    let character = get_character(id.clone())
        .await?
        .ok_or_else(|| ServerFnError::new("No such character"))?;

    let (hp_current, hp_temp) = apply_hp_delta(
        character.hp_current,
        character.hp_temp,
        character.hp_max,
        delta,
    );

    sqlx::query("UPDATE characters SET hp_current = ?, hp_temp = ?, updated_at = ? WHERE id = ?")
        .bind(hp_current)
        .bind(hp_temp)
        .bind(db::now())
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;

    Ok(())
}

/// Damage eats temporary hit points first; healing never exceeds max HP and
/// never restores temp HP. Pure, so it is unit tested below.
pub fn apply_hp_delta(hp_current: i64, hp_temp: i64, hp_max: i64, delta: i64) -> (i64, i64) {
    if delta >= 0 {
        return ((hp_current + delta).min(hp_max), hp_temp);
    }

    let mut damage = -delta;
    let absorbed = damage.min(hp_temp);
    damage -= absorbed;

    ((hp_current - damage).max(0), hp_temp - absorbed)
}

/// Toggle a value's presence in a JSON array column.
#[cfg(feature = "ssr")]
fn toggled_json_array(raw: &str, value: &str) -> String {
    let mut keys: Vec<String> = serde_json::from_str(raw).unwrap_or_default();
    match keys.iter().position(|k| k == value) {
        Some(idx) => {
            keys.remove(idx);
        }
        None => keys.push(value.to_string()),
    }
    serde_json::to_string(&keys).unwrap_or_else(|_| "[]".to_string())
}

#[server(ToggleSkillProficiency, "/api")]
pub async fn toggle_skill_proficiency(
    id: String,
    skill_key: String,
    expertise: bool,
) -> Result<(), ServerFnError> {
    use crate::db;

    let character = get_character(id.clone())
        .await?
        .ok_or_else(|| ServerFnError::new("No such character"))?;

    let column = if expertise {
        "skill_expertise"
    } else {
        "skill_proficiencies"
    };
    let current = if expertise {
        &character.skill_expertise
    } else {
        &character.skill_proficiencies
    };

    let updated = toggled_json_array(current, &skill_key);

    // Column name is chosen from a fixed pair above, never from user input.
    let sql = format!("UPDATE characters SET {column} = ?, updated_at = ? WHERE id = ?");
    sqlx::query(&sql)
        .bind(updated)
        .bind(db::now())
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;

    Ok(())
}

#[server(ToggleSaveProficiency, "/api")]
pub async fn toggle_save_proficiency(
    id: String,
    ability_key: String,
) -> Result<(), ServerFnError> {
    use crate::db;

    let character = get_character(id.clone())
        .await?
        .ok_or_else(|| ServerFnError::new("No such character"))?;

    let updated = toggled_json_array(&character.save_proficiencies, &ability_key);

    sqlx::query("UPDATE characters SET save_proficiencies = ?, updated_at = ? WHERE id = ?")
        .bind(updated)
        .bind(db::now())
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;

    Ok(())
}

/// A long rest restores all HP, clears temp HP and resets death saves.
#[server(LongRest, "/api")]
pub async fn long_rest(id: String) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query(
        "UPDATE characters
            SET hp_current = hp_max, hp_temp = 0,
                death_save_successes = 0, death_save_failures = 0,
                updated_at = ?
          WHERE id = ?",
    )
    .bind(db::now())
    .bind(&id)
    .execute(db::pool())
    .await
    .map_err(super::to_server_err)?;

    Ok(())
}

#[server(SetDeathSaves, "/api")]
pub async fn set_death_saves(
    id: String,
    successes: i64,
    failures: i64,
) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query(
        "UPDATE characters SET death_save_successes = ?, death_save_failures = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(successes.clamp(0, 3))
    .bind(failures.clamp(0, 3))
    .bind(db::now())
    .bind(&id)
    .execute(db::pool())
    .await
    .map_err(super::to_server_err)?;

    Ok(())
}

#[server(ToggleInspiration, "/api")]
pub async fn toggle_inspiration(id: String) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query(
        "UPDATE characters SET inspiration = 1 - inspiration, updated_at = ? WHERE id = ?",
    )
    .bind(db::now())
    .bind(&id)
    .execute(db::pool())
    .await
    .map_err(super::to_server_err)?;
    Ok(())
}

// ---------------------------------------------------------------- inventory

#[server(ListItems, "/api")]
pub async fn list_items(character_id: String) -> Result<Vec<Item>, ServerFnError> {
    use crate::db;

    sqlx::query_as::<_, Item>(
        "SELECT * FROM items WHERE character_id = ? ORDER BY equipped DESC, name COLLATE NOCASE",
    )
    .bind(&character_id)
    .fetch_all(db::pool())
    .await
    .map_err(super::to_server_err)
}

#[server(AddItem, "/api")]
pub async fn add_item(
    character_id: String,
    name: String,
    quantity: i64,
    weight: f64,
    description: String,
) -> Result<(), ServerFnError> {
    use crate::db;

    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(ServerFnError::new("Item needs a name"));
    }

    sqlx::query(
        "INSERT INTO items (id, character_id, name, quantity, weight, description, equipped)
         VALUES (?, ?, ?, ?, ?, ?, 0)",
    )
    .bind(db::new_id())
    .bind(&character_id)
    .bind(&name)
    .bind(quantity.max(1))
    .bind(weight.max(0.0))
    .bind(description.trim())
    .execute(db::pool())
    .await
    .map_err(super::to_server_err)?;

    Ok(())
}

#[server(ToggleEquipped, "/api")]
pub async fn toggle_equipped(item_id: String) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query("UPDATE items SET equipped = 1 - equipped WHERE id = ?")
        .bind(&item_id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
    Ok(())
}

#[server(DeleteItem, "/api")]
pub async fn delete_item(item_id: String) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query("DELETE FROM items WHERE id = ?")
        .bind(&item_id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healing_caps_at_max() {
        assert_eq!(apply_hp_delta(5, 0, 10, 100), (10, 0));
    }

    #[test]
    fn damage_consumes_temp_hp_first() {
        // 5 damage against 3 temp: temp gone, 2 carries through to real HP.
        assert_eq!(apply_hp_delta(10, 3, 10, -5), (8, 0));
    }

    #[test]
    fn temp_hp_can_fully_absorb() {
        assert_eq!(apply_hp_delta(10, 8, 10, -5), (10, 3));
    }

    #[test]
    fn hp_floors_at_zero() {
        assert_eq!(apply_hp_delta(3, 0, 10, -99), (0, 0));
    }

    #[test]
    fn healing_does_not_add_temp_hp() {
        assert_eq!(apply_hp_delta(4, 2, 10, 3), (7, 2));
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn toggling_adds_then_removes() {
        let once = toggled_json_array("[]", "stealth");
        assert_eq!(once, r#"["stealth"]"#);
        assert_eq!(toggled_json_array(&once, "stealth"), "[]");
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn toggling_survives_corrupt_json() {
        assert_eq!(toggled_json_array("garbage", "arcana"), r#"["arcana"]"#);
    }
}
