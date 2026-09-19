# D&D Dashboard

A dungeon master's dashboard for running a campaign: character sheets, a live
initiative tracker, an NPC/world codex, and a quest and session journal.

Everything is Rust — the frontend too. [Leptos](https://leptos.dev) compiles the
UI to WebAssembly and renders it server-side first, so pages arrive as HTML and
then hydrate. There is no JavaScript and no hand-written REST layer: the browser
calls the server through `#[server]` functions, which are ordinary Rust `async fn`s.

```
Browser (wasm)  ──  #[server] fn  ──  Axum  ──  SQLite
    Leptos            typed call       SSR      sqlx
```

## Requirements

- Rust stable (this was built against 1.98.1)
- The `wasm32-unknown-unknown` target
- `cargo-leptos`

```sh
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --locked   # or grab a prebuilt binary from its releases
```

## Running it

```sh
cargo leptos watch        # dev server with rebuild-on-save
cargo leptos serve        # build once and serve
```

Then open <http://127.0.0.1:3000>.

The database is created automatically at `data/dnd.db` on first run and the
migrations in `migrations/` are applied. Point `DATABASE_URL` somewhere else if
you like:

```sh
DATABASE_URL=sqlite:///srv/campaigns/curse-of-strahd.db cargo leptos serve
```

To start a fresh campaign, stop the server and delete `data/`.

## Tests

```sh
cargo test --no-default-features --features ssr
```

The suite covers the parts where being wrong is quiet and costly: 5e modifier
and proficiency math, expertise, XP thresholds, dice-notation parsing, initiative
sorting and its tiebreaks, turn/round wrap-around, and the temp-HP damage rules.

## What's in it

**Characters** — Full 5e sheets. Ability scores with derived modifiers, saving
throws, all 18 skills with proficiency and expertise pips, passive scores, AC,
speed, hit dice, inspiration, and inventory with carried weight. Damage and
healing respect temporary hit points; dropping to 0 HP reveals death saves. Long
rest restores the lot.

**Combat** — An initiative tracker. Pull the whole party in with one click, add
monsters, roll initiative for everyone at once (d20 + Dex), and step through
turns with automatic round counting. Damage applied here writes back to the
character's sheet, so HP survives the encounter. Conditions and concentration
are one tap each.

**World** — Locations, factions and NPCs, each with an attitude-toward-the-party
slider that buckets into Hostile → Devoted. NPCs carry a stat line, a public
description, and a DM-only secrets block.

**Journal** — Quests with status (active, rumor, completed, failed), a session
log that numbers itself, party gold, and party-wide XP awards.

**Dice** — `d20`, `2d6+3`, `4d6`, `1d8-1`. Rolls happen server-side, so no RNG
entropy source is needed in the wasm bundle. Natural 20s and 1s are called out.

## Layout

| Path | What lives there |
| --- | --- |
| `src/rules.rs` | 5e math: modifiers, proficiency, skills, XP. Pure, shared, tested. |
| `src/dice.rs` | Dice notation parser (shared) and roller (server-only). |
| `src/models.rs` | Domain types, shared between server and browser. |
| `src/server/` | `#[server]` functions — the API surface. |
| `src/pages/` | One module per route. |
| `src/components/` | Reusable UI (HP bar, dice roller). |
| `src/auth.rs` | Authorization seam — see below. |
| `migrations/` | SQL schema. |

## On authentication

It runs local and single-user: no login, one SQLite file. But it was built so
adding auth later isn't a rewrite. Every table that needs one already carries a
nullable `owner_id`, and every permission question goes through `src/auth.rs`:

```rust
pub enum Actor {
    Dm,
    Player { user_id: String },
}
```

`current_actor()` returns `Actor::Dm` today. To open it up to players, give that
function a session cookie to read, add a `users` table, and make the list queries
filter on `owner_id` — the call sites and the `can_edit` / `sees_secrets` checks
are already in place and unit tested.
