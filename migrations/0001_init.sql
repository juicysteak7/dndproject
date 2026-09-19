-- D&D Dashboard initial schema.
--
-- Auth seam: `owner_id` columns are NULL today (single-DM local mode). When
-- auth is added they become FKs to a `users` table, and the visibility helpers
-- in src/auth.rs start filtering on them instead of returning everything.

CREATE TABLE parties (
    id          TEXT PRIMARY KEY,
    owner_id    TEXT,
    name        TEXT    NOT NULL,
    notes       TEXT    NOT NULL DEFAULT '',
    gold        INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT    NOT NULL
);

CREATE TABLE characters (
    id          TEXT PRIMARY KEY,
    party_id    TEXT REFERENCES parties(id) ON DELETE SET NULL,
    owner_id    TEXT,
    name        TEXT    NOT NULL,
    player_name TEXT    NOT NULL DEFAULT '',
    class       TEXT    NOT NULL DEFAULT '',
    subclass    TEXT    NOT NULL DEFAULT '',
    race        TEXT    NOT NULL DEFAULT '',
    background  TEXT    NOT NULL DEFAULT '',
    alignment   TEXT    NOT NULL DEFAULT '',
    level       INTEGER NOT NULL DEFAULT 1,
    xp          INTEGER NOT NULL DEFAULT 0,

    str_score   INTEGER NOT NULL DEFAULT 10,
    dex_score   INTEGER NOT NULL DEFAULT 10,
    con_score   INTEGER NOT NULL DEFAULT 10,
    int_score   INTEGER NOT NULL DEFAULT 10,
    wis_score   INTEGER NOT NULL DEFAULT 10,
    cha_score   INTEGER NOT NULL DEFAULT 10,

    hp_max      INTEGER NOT NULL DEFAULT 1,
    hp_current  INTEGER NOT NULL DEFAULT 1,
    hp_temp     INTEGER NOT NULL DEFAULT 0,
    armor_class INTEGER NOT NULL DEFAULT 10,
    speed       INTEGER NOT NULL DEFAULT 30,
    hit_dice    TEXT    NOT NULL DEFAULT '',

    death_save_successes INTEGER NOT NULL DEFAULT 0,
    death_save_failures  INTEGER NOT NULL DEFAULT 0,
    inspiration          INTEGER NOT NULL DEFAULT 0,

    -- JSON arrays of skill/ability keys (see src/rules.rs)
    skill_proficiencies TEXT NOT NULL DEFAULT '[]',
    skill_expertise     TEXT NOT NULL DEFAULT '[]',
    save_proficiencies  TEXT NOT NULL DEFAULT '[]',

    notes       TEXT    NOT NULL DEFAULT '',
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT    NOT NULL,
    updated_at  TEXT    NOT NULL
);

CREATE INDEX idx_characters_party ON characters(party_id);

CREATE TABLE items (
    id           TEXT PRIMARY KEY,
    character_id TEXT    NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    name         TEXT    NOT NULL,
    quantity     INTEGER NOT NULL DEFAULT 1,
    weight       REAL    NOT NULL DEFAULT 0,
    description  TEXT    NOT NULL DEFAULT '',
    equipped     INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_items_character ON items(character_id);

CREATE TABLE locations (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    kind        TEXT NOT NULL DEFAULT 'settlement',
    parent_id   TEXT REFERENCES locations(id) ON DELETE SET NULL,
    description TEXT NOT NULL DEFAULT '',
    discovered  INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE factions (
    id          TEXT PRIMARY KEY,
    name        TEXT    NOT NULL,
    description TEXT    NOT NULL DEFAULT '',
    attitude    INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE npcs (
    id               TEXT PRIMARY KEY,
    name             TEXT    NOT NULL,
    race             TEXT    NOT NULL DEFAULT '',
    role             TEXT    NOT NULL DEFAULT '',
    location_id      TEXT REFERENCES locations(id) ON DELETE SET NULL,
    faction_id       TEXT REFERENCES factions(id) ON DELETE SET NULL,
    attitude         INTEGER NOT NULL DEFAULT 0,
    armor_class      INTEGER NOT NULL DEFAULT 10,
    hp_max           INTEGER NOT NULL DEFAULT 1,
    challenge_rating TEXT    NOT NULL DEFAULT '',
    description      TEXT    NOT NULL DEFAULT '',
    secrets          TEXT    NOT NULL DEFAULT '',
    alive            INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX idx_npcs_location ON npcs(location_id);
CREATE INDEX idx_npcs_faction ON npcs(faction_id);

CREATE TABLE quests (
    id           TEXT PRIMARY KEY,
    party_id     TEXT REFERENCES parties(id) ON DELETE CASCADE,
    title        TEXT NOT NULL,
    status       TEXT NOT NULL DEFAULT 'active',
    description  TEXT NOT NULL DEFAULT '',
    reward       TEXT NOT NULL DEFAULT '',
    giver_npc_id TEXT REFERENCES npcs(id) ON DELETE SET NULL,
    location_id  TEXT REFERENCES locations(id) ON DELETE SET NULL,
    created_at   TEXT NOT NULL,
    completed_at TEXT
);

CREATE INDEX idx_quests_party ON quests(party_id);

CREATE TABLE game_sessions (
    id             TEXT PRIMARY KEY,
    party_id       TEXT REFERENCES parties(id) ON DELETE CASCADE,
    session_number INTEGER NOT NULL,
    played_on      TEXT    NOT NULL,
    title          TEXT    NOT NULL,
    summary        TEXT    NOT NULL DEFAULT '',
    notes          TEXT    NOT NULL DEFAULT '',
    xp_awarded     INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_sessions_party ON game_sessions(party_id);

CREATE TABLE encounters (
    id         TEXT PRIMARY KEY,
    party_id   TEXT REFERENCES parties(id) ON DELETE CASCADE,
    name       TEXT    NOT NULL,
    round      INTEGER NOT NULL DEFAULT 1,
    turn_index INTEGER NOT NULL DEFAULT 0,
    status     TEXT    NOT NULL DEFAULT 'planning',
    created_at TEXT    NOT NULL
);

CREATE TABLE combatants (
    id             TEXT PRIMARY KEY,
    encounter_id   TEXT    NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    character_id   TEXT REFERENCES characters(id) ON DELETE SET NULL,
    name           TEXT    NOT NULL,
    initiative     INTEGER NOT NULL DEFAULT 0,
    dex_score      INTEGER NOT NULL DEFAULT 10,
    hp_max         INTEGER NOT NULL DEFAULT 1,
    hp_current     INTEGER NOT NULL DEFAULT 1,
    hp_temp        INTEGER NOT NULL DEFAULT 0,
    armor_class    INTEGER NOT NULL DEFAULT 10,
    conditions     TEXT    NOT NULL DEFAULT '[]',
    concentrating  INTEGER NOT NULL DEFAULT 0,
    is_pc          INTEGER NOT NULL DEFAULT 0,
    notes          TEXT    NOT NULL DEFAULT ''
);

CREATE INDEX idx_combatants_encounter ON combatants(encounter_id);
