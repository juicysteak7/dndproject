use leptos::prelude::*;

use crate::models::{Faction, Location, Npc};

// ---------------------------------------------------------------- locations

#[server(ListLocations, "/api")]
pub async fn list_locations() -> Result<Vec<Location>, ServerFnError> {
    use crate::db;

    sqlx::query_as::<_, Location>("SELECT * FROM locations ORDER BY name COLLATE NOCASE")
        .fetch_all(db::pool())
        .await
        .map_err(super::to_server_err)
}

#[server(CreateLocation, "/api")]
pub async fn create_location(
    name: String,
    kind: String,
    description: String,
) -> Result<(), ServerFnError> {
    use crate::db;

    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(ServerFnError::new("Location needs a name"));
    }

    sqlx::query(
        "INSERT INTO locations (id, name, kind, parent_id, description, discovered)
         VALUES (?, ?, ?, NULL, ?, 1)",
    )
    .bind(db::new_id())
    .bind(&name)
    .bind(kind.trim())
    .bind(description.trim())
    .execute(db::pool())
    .await
    .map_err(super::to_server_err)?;

    Ok(())
}

#[server(DeleteLocation, "/api")]
pub async fn delete_location(id: String) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query("DELETE FROM locations WHERE id = ?")
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
    Ok(())
}

// ----------------------------------------------------------------- factions

#[server(ListFactions, "/api")]
pub async fn list_factions() -> Result<Vec<Faction>, ServerFnError> {
    use crate::db;

    sqlx::query_as::<_, Faction>("SELECT * FROM factions ORDER BY name COLLATE NOCASE")
        .fetch_all(db::pool())
        .await
        .map_err(super::to_server_err)
}

#[server(CreateFaction, "/api")]
pub async fn create_faction(
    name: String,
    description: String,
    attitude: i64,
) -> Result<(), ServerFnError> {
    use crate::db;

    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(ServerFnError::new("Faction needs a name"));
    }

    sqlx::query("INSERT INTO factions (id, name, description, attitude) VALUES (?, ?, ?, ?)")
        .bind(db::new_id())
        .bind(&name)
        .bind(description.trim())
        .bind(attitude.clamp(-100, 100))
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;

    Ok(())
}

#[server(SetFactionAttitude, "/api")]
pub async fn set_faction_attitude(id: String, attitude: i64) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query("UPDATE factions SET attitude = ? WHERE id = ?")
        .bind(attitude.clamp(-100, 100))
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
    Ok(())
}

#[server(DeleteFaction, "/api")]
pub async fn delete_faction(id: String) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query("DELETE FROM factions WHERE id = ?")
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
    Ok(())
}

// --------------------------------------------------------------------- NPCs

#[server(ListNpcs, "/api")]
pub async fn list_npcs() -> Result<Vec<Npc>, ServerFnError> {
    use crate::db;

    sqlx::query_as::<_, Npc>("SELECT * FROM npcs ORDER BY name COLLATE NOCASE")
        .fetch_all(db::pool())
        .await
        .map_err(super::to_server_err)
}

#[server(CreateNpc, "/api")]
#[allow(clippy::too_many_arguments)]
pub async fn create_npc(
    name: String,
    race: String,
    role: String,
    location_id: String,
    faction_id: String,
    attitude: i64,
    armor_class: i64,
    hp_max: i64,
    challenge_rating: String,
    description: String,
    secrets: String,
) -> Result<(), ServerFnError> {
    use crate::db;

    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(ServerFnError::new("NPC needs a name"));
    }

    // Empty select values mean "unassigned", which the schema stores as NULL.
    let location_id = blank_to_none(location_id);
    let faction_id = blank_to_none(faction_id);

    sqlx::query(
        "INSERT INTO npcs
            (id, name, race, role, location_id, faction_id, attitude,
             armor_class, hp_max, challenge_rating, description, secrets, alive)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1)",
    )
    .bind(db::new_id())
    .bind(&name)
    .bind(race.trim())
    .bind(role.trim())
    .bind(location_id)
    .bind(faction_id)
    .bind(attitude.clamp(-100, 100))
    .bind(armor_class.max(0))
    .bind(hp_max.max(1))
    .bind(challenge_rating.trim())
    .bind(description.trim())
    .bind(secrets.trim())
    .execute(db::pool())
    .await
    .map_err(super::to_server_err)?;

    Ok(())
}

/// An empty or whitespace-only form value means "not set".
pub fn blank_to_none(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[server(SetNpcAttitude, "/api")]
pub async fn set_npc_attitude(id: String, attitude: i64) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query("UPDATE npcs SET attitude = ? WHERE id = ?")
        .bind(attitude.clamp(-100, 100))
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
    Ok(())
}

#[server(ToggleNpcAlive, "/api")]
pub async fn toggle_npc_alive(id: String) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query("UPDATE npcs SET alive = 1 - alive WHERE id = ?")
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
    Ok(())
}

#[server(DeleteNpc, "/api")]
pub async fn delete_npc(id: String) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query("DELETE FROM npcs WHERE id = ?")
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::blank_to_none;

    #[test]
    fn blank_selects_become_null() {
        assert_eq!(blank_to_none(String::new()), None);
        assert_eq!(blank_to_none("   ".into()), None);
        assert_eq!(blank_to_none(" abc ".into()), Some("abc".to_string()));
    }
}
