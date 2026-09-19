use leptos::prelude::*;

use crate::models::{GameSession, Party, Quest};

// ------------------------------------------------------------------- quests

#[server(ListQuests, "/api")]
pub async fn list_quests() -> Result<Vec<Quest>, ServerFnError> {
    use crate::db;

    // Active work first, then rumours, then anything already resolved.
    sqlx::query_as::<_, Quest>(
        "SELECT * FROM quests
         ORDER BY CASE status
                    WHEN 'active' THEN 0
                    WHEN 'rumor' THEN 1
                    WHEN 'completed' THEN 2
                    ELSE 3
                  END,
                  created_at DESC",
    )
    .fetch_all(db::pool())
    .await
    .map_err(super::to_server_err)
}

#[server(CreateQuest, "/api")]
pub async fn create_quest(
    title: String,
    description: String,
    reward: String,
    status: String,
) -> Result<(), ServerFnError> {
    use crate::db;

    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(ServerFnError::new("Quest needs a title"));
    }

    let status = normalize_status(&status);
    let party_id = db::ensure_default_party()
        .await
        .map_err(super::to_server_err)?;

    sqlx::query(
        "INSERT INTO quests
            (id, party_id, title, status, description, reward,
             giver_npc_id, location_id, created_at, completed_at)
         VALUES (?, ?, ?, ?, ?, ?, NULL, NULL, ?, NULL)",
    )
    .bind(db::new_id())
    .bind(&party_id)
    .bind(&title)
    .bind(&status)
    .bind(description.trim())
    .bind(reward.trim())
    .bind(db::now())
    .execute(db::pool())
    .await
    .map_err(super::to_server_err)?;

    Ok(())
}

/// Keep unknown values out of the status column.
pub fn normalize_status(status: &str) -> String {
    let lowered = status.trim().to_ascii_lowercase();
    if crate::models::QUEST_STATUSES.contains(&lowered.as_str()) {
        lowered
    } else {
        "active".to_string()
    }
}

#[server(SetQuestStatus, "/api")]
pub async fn set_quest_status(id: String, status: String) -> Result<(), ServerFnError> {
    use crate::db;

    let status = normalize_status(&status);
    // Stamp a completion date when the quest leaves the active list.
    let completed_at = if status == "active" || status == "rumor" {
        None
    } else {
        Some(db::now())
    };

    sqlx::query("UPDATE quests SET status = ?, completed_at = ? WHERE id = ?")
        .bind(&status)
        .bind(completed_at)
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;

    Ok(())
}

#[server(DeleteQuest, "/api")]
pub async fn delete_quest(id: String) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query("DELETE FROM quests WHERE id = ?")
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
    Ok(())
}

// ------------------------------------------------------------ session log

#[server(ListSessions, "/api")]
pub async fn list_sessions() -> Result<Vec<GameSession>, ServerFnError> {
    use crate::db;

    sqlx::query_as::<_, GameSession>(
        "SELECT * FROM game_sessions ORDER BY session_number DESC",
    )
    .fetch_all(db::pool())
    .await
    .map_err(super::to_server_err)
}

#[server(CreateSession, "/api")]
pub async fn create_session(
    title: String,
    played_on: String,
    summary: String,
    notes: String,
    xp_awarded: i64,
) -> Result<(), ServerFnError> {
    use crate::db;

    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(ServerFnError::new("Session needs a title"));
    }

    let party_id = db::ensure_default_party()
        .await
        .map_err(super::to_server_err)?;

    // Number sessions automatically so the DM never has to track the count.
    let next: (i64,) =
        sqlx::query_as("SELECT COALESCE(MAX(session_number), 0) + 1 FROM game_sessions")
            .fetch_one(db::pool())
            .await
            .map_err(super::to_server_err)?;

    let played_on = if played_on.trim().is_empty() {
        db::today()
    } else {
        played_on.trim().to_string()
    };

    sqlx::query(
        "INSERT INTO game_sessions
            (id, party_id, session_number, played_on, title, summary, notes, xp_awarded)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(db::new_id())
    .bind(&party_id)
    .bind(next.0)
    .bind(&played_on)
    .bind(&title)
    .bind(summary.trim())
    .bind(notes.trim())
    .bind(xp_awarded.max(0))
    .execute(db::pool())
    .await
    .map_err(super::to_server_err)?;

    Ok(())
}

#[server(DeleteSession, "/api")]
pub async fn delete_session(id: String) -> Result<(), ServerFnError> {
    use crate::db;

    sqlx::query("DELETE FROM game_sessions WHERE id = ?")
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;
    Ok(())
}

/// Award XP to every active party member at once.
#[server(AwardXp, "/api")]
pub async fn award_xp(amount: i64) -> Result<(), ServerFnError> {
    use crate::db;

    if amount == 0 {
        return Ok(());
    }

    sqlx::query(
        "UPDATE characters SET xp = MAX(0, xp + ?), updated_at = ? WHERE is_active = 1",
    )
    .bind(amount)
    .bind(db::now())
    .execute(db::pool())
    .await
    .map_err(super::to_server_err)?;

    Ok(())
}

// -------------------------------------------------------------------- party

#[server(GetParty, "/api")]
pub async fn get_party() -> Result<Party, ServerFnError> {
    use crate::db;

    let id = db::ensure_default_party()
        .await
        .map_err(super::to_server_err)?;

    sqlx::query_as::<_, Party>("SELECT * FROM parties WHERE id = ?")
        .bind(&id)
        .fetch_one(db::pool())
        .await
        .map_err(super::to_server_err)
}

#[server(AdjustGold, "/api")]
pub async fn adjust_gold(delta: i64) -> Result<(), ServerFnError> {
    use crate::db;

    let id = db::ensure_default_party()
        .await
        .map_err(super::to_server_err)?;

    sqlx::query("UPDATE parties SET gold = MAX(0, gold + ?) WHERE id = ?")
        .bind(delta)
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;

    Ok(())
}

#[server(SetPartyNotes, "/api")]
pub async fn set_party_notes(notes: String) -> Result<(), ServerFnError> {
    use crate::db;

    let id = db::ensure_default_party()
        .await
        .map_err(super::to_server_err)?;

    sqlx::query("UPDATE parties SET notes = ? WHERE id = ?")
        .bind(notes)
        .bind(&id)
        .execute(db::pool())
        .await
        .map_err(super::to_server_err)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::normalize_status;

    #[test]
    fn known_statuses_pass_through() {
        for s in ["active", "completed", "failed", "rumor"] {
            assert_eq!(normalize_status(s), s);
        }
    }

    #[test]
    fn status_is_case_insensitive() {
        assert_eq!(normalize_status("  CoMpLeTeD "), "completed");
    }

    #[test]
    fn unknown_statuses_fall_back_to_active() {
        assert_eq!(normalize_status("nonsense"), "active");
        assert_eq!(normalize_status(""), "active");
    }
}
