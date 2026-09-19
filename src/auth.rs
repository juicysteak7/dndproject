//! Authorization seam.
//!
//! The dashboard runs local and single-user today, so `current_actor` always
//! returns the DM and the visibility helpers are permissive. The call sites are
//! already threaded through every server function, so adding real auth means
//! changing this file (plus a `users` table) rather than touching every query.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Actor {
    /// Full read/write across the whole campaign.
    Dm,
    /// Can read shared world info and write only their own character.
    Player { user_id: String },
}

impl Actor {
    pub fn is_dm(&self) -> bool {
        matches!(self, Actor::Dm)
    }

    /// The `owner_id` this actor writes as, if any.
    pub fn owner_id(&self) -> Option<&str> {
        match self {
            Actor::Dm => None,
            Actor::Player { user_id } => Some(user_id),
        }
    }

    /// Whether this actor may edit a record with the given owner.
    pub fn can_edit(&self, owner_id: Option<&str>) -> bool {
        match self {
            Actor::Dm => true,
            Actor::Player { user_id } => owner_id == Some(user_id.as_str()),
        }
    }

    /// Whether this actor may see DM-only fields (NPC secrets, undiscovered
    /// locations, hidden quest notes).
    pub fn sees_secrets(&self) -> bool {
        self.is_dm()
    }
}

/// Today: always the DM. Later: read the session cookie.
#[cfg(feature = "ssr")]
pub fn current_actor() -> Actor {
    Actor::Dm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dm_edits_anything() {
        assert!(Actor::Dm.can_edit(None));
        assert!(Actor::Dm.can_edit(Some("someone")));
        assert!(Actor::Dm.sees_secrets());
    }

    #[test]
    fn player_edits_only_their_own() {
        let player = Actor::Player {
            user_id: "alex".into(),
        };
        assert!(player.can_edit(Some("alex")));
        assert!(!player.can_edit(Some("sam")));
        assert!(!player.can_edit(None));
        assert!(!player.sees_secrets());
    }
}
