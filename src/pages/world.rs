use leptos::prelude::*;

use crate::models::attitude_label;
use crate::server::world::{
    list_factions, list_locations, list_npcs, CreateFaction, CreateLocation, CreateNpc,
    DeleteFaction, DeleteLocation, DeleteNpc, SetFactionAttitude, SetNpcAttitude, ToggleNpcAlive,
};

#[component]
pub fn WorldPage() -> impl IntoView {
    let create_location = ServerAction::<CreateLocation>::new();
    let delete_location = ServerAction::<DeleteLocation>::new();
    let create_faction = ServerAction::<CreateFaction>::new();
    let delete_faction = ServerAction::<DeleteFaction>::new();
    let faction_attitude = ServerAction::<SetFactionAttitude>::new();
    let create_npc = ServerAction::<CreateNpc>::new();
    let delete_npc = ServerAction::<DeleteNpc>::new();
    let npc_attitude = ServerAction::<SetNpcAttitude>::new();
    let toggle_alive = ServerAction::<ToggleNpcAlive>::new();

    let locations = Resource::new(
        move || (create_location.version().get(), delete_location.version().get()),
        |_| list_locations(),
    );
    let factions = Resource::new(
        move || {
            (
                create_faction.version().get(),
                delete_faction.version().get(),
                faction_attitude.version().get(),
            )
        },
        |_| list_factions(),
    );
    let npcs = Resource::new(
        move || {
            (
                create_npc.version().get(),
                delete_npc.version().get(),
                npc_attitude.version().get(),
                toggle_alive.version().get(),
            )
        },
        |_| list_npcs(),
    );
    // The NPC form needs the current lists to populate its selects.
    let location_options = Resource::new(
        move || create_location.version().get(),
        |_| list_locations(),
    );
    let faction_options = Resource::new(
        move || create_faction.version().get(),
        |_| list_factions(),
    );

    view! {
        <h1>"World"</h1>

        <div class="grid">
            <section class="card">
                <h3>"Locations"</h3>
                <ActionForm action=create_location attr:class="form-row">
                    <input type="text" name="name" placeholder="Name" required />
                    <input type="text" name="kind" placeholder="Kind (city, dungeon…)" />
                    <input type="text" name="description" placeholder="Description" />
                    <button type="submit">"Add"</button>
                </ActionForm>

                <Suspense fallback=move || view! { <p>"Loading…"</p> }>
                    {move || {
                        locations
                            .get()
                            .map(|result| match result {
                                Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                                Ok(list) if list.is_empty() => {
                                    view! { <p class="muted">"Nowhere mapped yet."</p> }.into_any()
                                }
                                Ok(list) => {
                                    view! {
                                        <ul class="plain">
                                            {list
                                                .into_iter()
                                                .map(|l| {
                                                    let id = l.id.clone();
                                                    view! {
                                                        <li class="list-row">
                                                            <div>
                                                                <strong>{l.name.clone()}</strong>
                                                                <span class="muted">" · " {l.kind.clone()}</span>
                                                                <p class="muted">{l.description.clone()}</p>
                                                            </div>
                                                            <button
                                                                class="chip danger"
                                                                on:click=move |_| {
                                                                    delete_location.dispatch(DeleteLocation { id: id.clone() });
                                                                }
                                                            >
                                                                "×"
                                                            </button>
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
                <h3>"Factions"</h3>
                <ActionForm action=create_faction attr:class="form-row">
                    <input type="text" name="name" placeholder="Name" required />
                    <input type="text" name="description" placeholder="Description" />
                    <input
                        type="number"
                        name="attitude"
                        value="0"
                        min="-100"
                        max="100"
                        title="Attitude toward the party"
                    />
                    <button type="submit">"Add"</button>
                </ActionForm>

                <Suspense fallback=move || view! { <p>"Loading…"</p> }>
                    {move || {
                        factions
                            .get()
                            .map(|result| match result {
                                Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                                Ok(list) if list.is_empty() => {
                                    view! { <p class="muted">"No factions yet."</p> }.into_any()
                                }
                                Ok(list) => {
                                    view! {
                                        <ul class="plain">
                                            {list
                                                .into_iter()
                                                .map(|f| {
                                                    let (id, del_id) = (f.id.clone(), f.id.clone());
                                                    let attitude = f.attitude;
                                                    view! {
                                                        <li class="list-row">
                                                            <div>
                                                                <strong>{f.name.clone()}</strong>
                                                                <span class=format!(
                                                                    "badge {}",
                                                                    attitude_label(attitude).to_lowercase(),
                                                                )>{attitude_label(attitude)}</span>
                                                                <p class="muted">{f.description.clone()}</p>
                                                                <input
                                                                    type="range"
                                                                    min="-100"
                                                                    max="100"
                                                                    prop:value=attitude
                                                                    on:change=move |ev| {
                                                                        if let Ok(v) = event_target_value(&ev).parse::<i64>() {
                                                                            faction_attitude
                                                                                .dispatch(SetFactionAttitude {
                                                                                    id: id.clone(),
                                                                                    attitude: v,
                                                                                });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                            <button
                                                                class="chip danger"
                                                                on:click=move |_| {
                                                                    delete_faction.dispatch(DeleteFaction { id: del_id.clone() });
                                                                }
                                                            >
                                                                "×"
                                                            </button>
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
        </div>

        <section class="card">
            <h3>"NPCs"</h3>
            <Suspense fallback=move || view! { <p>"Loading…"</p> }>
                {move || {
                    let locs = location_options.get().and_then(|r| r.ok()).unwrap_or_default();
                    let facs = faction_options.get().and_then(|r| r.ok()).unwrap_or_default();
                    view! {
                        <ActionForm action=create_npc attr:class="form-row wrap">
                            <input type="text" name="name" placeholder="Name" required />
                            <input type="text" name="race" placeholder="Race" />
                            <input type="text" name="role" placeholder="Role" />
                            <select name="location_id">
                                <option value="">"— location —"</option>
                                {locs
                                    .into_iter()
                                    .map(|l| {
                                        view! { <option value=l.id.clone()>{l.name.clone()}</option> }
                                    })
                                    .collect::<Vec<_>>()}
                            </select>
                            <select name="faction_id">
                                <option value="">"— faction —"</option>
                                {facs
                                    .into_iter()
                                    .map(|f| {
                                        view! { <option value=f.id.clone()>{f.name.clone()}</option> }
                                    })
                                    .collect::<Vec<_>>()}
                            </select>
                            <input
                                type="number"
                                name="attitude"
                                value="0"
                                min="-100"
                                max="100"
                                title="Attitude"
                            />
                            <input type="number" name="armor_class" value="12" title="AC" />
                            <input type="number" name="hp_max" value="10" min="1" title="HP" />
                            <input type="text" name="challenge_rating" placeholder="CR" />
                            <input type="text" name="description" placeholder="Description" />
                            <input type="text" name="secrets" placeholder="Secrets (DM only)" />
                            <button type="submit">"Add NPC"</button>
                        </ActionForm>
                    }
                }}
            </Suspense>

            <Suspense fallback=move || view! { <p>"Loading…"</p> }>
                {move || {
                    npcs.get()
                        .map(|result| match result {
                            Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                            Ok(list) if list.is_empty() => {
                                view! { <p class="muted">"No NPCs yet."</p> }.into_any()
                            }
                            Ok(list) => {
                                view! {
                                    <div class="grid">
                                        {list
                                            .into_iter()
                                            .map(|n| {
                                                let (att_id, alive_id) = (n.id.clone(), n.id.clone());
                                                let del_id = n.id.clone();
                                                let attitude = n.attitude;
                                                let dead = n.alive == 0;
                                                view! {
                                                    <section class={if dead { "card npc dead" } else { "card npc" }}>
                                                        <div class="card-head">
                                                            <strong>{n.name.clone()}</strong>
                                                            <span class=format!(
                                                                "badge {}",
                                                                attitude_label(attitude).to_lowercase(),
                                                            )>{attitude_label(attitude)}</span>
                                                            {dead.then(|| view! { <span class="badge">"Deceased"</span> })}
                                                        </div>
                                                        <p class="muted">
                                                            {n.race.clone()} " " {n.role.clone()}
                                                            {(!n.challenge_rating.is_empty())
                                                                .then(|| format!(" · CR {}", n.challenge_rating))}
                                                        </p>
                                                        <p class="muted">"AC " {n.armor_class} " · HP " {n.hp_max}</p>
                                                        <p>{n.description.clone()}</p>
                                                        {(!n.secrets.is_empty())
                                                            .then(|| {
                                                                view! {
                                                                    <details class="secrets">
                                                                        <summary>"DM secrets"</summary>
                                                                        <p>{n.secrets.clone()}</p>
                                                                    </details>
                                                                }
                                                            })}
                                                        <input
                                                            type="range"
                                                            min="-100"
                                                            max="100"
                                                            prop:value=attitude
                                                            on:change=move |ev| {
                                                                if let Ok(v) = event_target_value(&ev).parse::<i64>() {
                                                                    npc_attitude
                                                                        .dispatch(SetNpcAttitude {
                                                                            id: att_id.clone(),
                                                                            attitude: v,
                                                                        });
                                                                }
                                                            }
                                                        />
                                                        <div class="row wrap">
                                                            <button
                                                                class="chip"
                                                                on:click=move |_| {
                                                                    toggle_alive.dispatch(ToggleNpcAlive { id: alive_id.clone() });
                                                                }
                                                            >
                                                                {if dead { "Revive" } else { "Mark dead" }}
                                                            </button>
                                                            <button
                                                                class="chip danger"
                                                                on:click=move |_| {
                                                                    delete_npc.dispatch(DeleteNpc { id: del_id.clone() });
                                                                }
                                                            >
                                                                "Delete"
                                                            </button>
                                                        </div>
                                                    </section>
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
    }
}
