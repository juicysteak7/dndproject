use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::dice_roller::DiceRoller;
use crate::components::hp_bar::HpBar;
use crate::rules::format_bonus;
use crate::server::characters::list_characters;
use crate::server::journal::{list_quests, list_sessions, AdjustGold};

#[component]
pub fn OverviewPage() -> impl IntoView {
    let adjust_gold = ServerAction::<AdjustGold>::new();

    let characters = Resource::new(|| (), |_| list_characters());
    let quests = Resource::new(|| (), |_| list_quests());
    let sessions = Resource::new(|| (), |_| list_sessions());
    let party = Resource::new(
        move || adjust_gold.version().get(),
        |_| crate::server::journal::get_party(),
    );

    view! {
        <h1>"Campaign Overview"</h1>

        <div class="grid">
            <section class="card">
                <h3>"Party"</h3>
                <Suspense fallback=move || view! { <p>"Loading…"</p> }>
                    {move || {
                        characters
                            .get()
                            .map(|result| match result {
                                Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                                Ok(list) if list.is_empty() => {
                                    view! {
                                        <p>
                                            "No characters yet. "
                                            <A href="/characters">"Add one"</A> "."
                                        </p>
                                    }
                                        .into_any()
                                }
                                Ok(list) => {
                                    let down = list.iter().filter(|c| c.is_unconscious()).count();
                                    let avg_level = list.iter().map(|c| c.level).sum::<i64>()
                                        / list.len() as i64;
                                    view! {
                                        <p class="muted">
                                            {list.len()} " characters · average level " {avg_level}
                                            {(down > 0).then(|| format!(" · {down} down"))}
                                        </p>
                                        <ul class="plain">
                                            {list
                                                .into_iter()
                                                .map(|c| {
                                                    let href = format!("/characters/{}", c.id);
                                                    // Read the fields out first; the link's children
                                                    // become a closure that would otherwise move `c`.
                                                    let name = c.name.clone();
                                                    let (hp_current, hp_max, hp_temp) = (
                                                        c.hp_current,
                                                        c.hp_max,
                                                        c.hp_temp,
                                                    );
                                                    let armor_class = c.armor_class;
                                                    let passive_perception = c.passive_perception();
                                                    view! {
                                                        <li class="party-row">
                                                            <A href=href>{name}</A>
                                                            <HpBar
                                                                current=hp_current
                                                                max=hp_max
                                                                temp=hp_temp
                                                            />
                                                            <span class="muted">
                                                                "AC " {armor_class} " · PP "
                                                                {passive_perception}
                                                            </span>
                                                        </li>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </ul>
                                    }
                                        .into_any()
                                }
                            })
                    }}
                </Suspense>
            </section>

            <section class="card">
                <h3>"Treasury"</h3>
                <Suspense fallback=move || view! { <p>"Loading…"</p> }>
                    {move || {
                        party
                            .get()
                            .map(|result| match result {
                                Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                                Ok(p) => {
                                    view! {
                                        <p class="stat-big">{p.gold} " gp"</p>
                                        <div class="row wrap">
                                            {[-100i64, -10, -1, 1, 10, 100]
                                                .into_iter()
                                                .map(|delta| {
                                                    view! {
                                                        <button
                                                            class="chip"
                                                            on:click=move |_| {
                                                                adjust_gold.dispatch(AdjustGold { delta });
                                                            }
                                                        >
                                                            {format_bonus(delta)}
                                                        </button>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </div>
                                    }
                                        .into_any()
                                }
                            })
                    }}
                </Suspense>
            </section>

            <DiceRoller />

            <section class="card">
                <h3>"Active quests"</h3>
                <Suspense fallback=move || view! { <p>"Loading…"</p> }>
                    {move || {
                        quests
                            .get()
                            .map(|result| match result {
                                Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                                Ok(list) => {
                                    let active: Vec<_> = list
                                        .into_iter()
                                        .filter(|q| q.status == "active")
                                        .collect();
                                    if active.is_empty() {
                                        view! { <p class="muted">"Nothing on the board."</p> }
                                            .into_any()
                                    } else {
                                        view! {
                                            <ul class="plain">
                                                {active
                                                    .into_iter()
                                                    .take(6)
                                                    .map(|q| {
                                                        view! { <li>{q.title.clone()}</li> }
                                                    })
                                                    .collect::<Vec<_>>()}
                                            </ul>
                                            <A href="/journal">"Open the journal"</A>
                                        }
                                            .into_any()
                                    }
                                }
                            })
                    }}
                </Suspense>
            </section>

            <section class="card">
                <h3>"Last session"</h3>
                <Suspense fallback=move || view! { <p>"Loading…"</p> }>
                    {move || {
                        sessions
                            .get()
                            .map(|result| match result {
                                Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                                Ok(list) => {
                                    match list.into_iter().next() {
                                        None => {
                                            view! { <p class="muted">"No sessions logged yet."</p> }
                                                .into_any()
                                        }
                                        Some(s) => {
                                            view! {
                                                <p>
                                                    <strong>
                                                        "#" {s.session_number} " " {s.title.clone()}
                                                    </strong>
                                                </p>
                                                <p class="muted">{s.played_on.clone()}</p>
                                                <p>{s.summary.clone()}</p>
                                            }
                                                .into_any()
                                        }
                                    }
                                }
                            })
                    }}
                </Suspense>
            </section>
        </div>
    }
}
