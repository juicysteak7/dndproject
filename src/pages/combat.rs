use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::components::hp_bar::HpBar;
use crate::rules::CONDITIONS;
use crate::server::combat::{
    get_encounter, list_combatants, list_encounters, AddCombatant, AddPartyToEncounter,
    AdjustCombatantHp, CreateEncounter, DeleteEncounter, EndEncounter, NextTurn, PreviousTurn,
    RemoveCombatant, RollAllInitiative, SetInitiative, ToggleCondition, ToggleConcentration,
};

#[component]
pub fn EncounterListPage() -> impl IntoView {
    let create = ServerAction::<CreateEncounter>::new();
    let delete = ServerAction::<DeleteEncounter>::new();

    let encounters = Resource::new(
        move || (create.version().get(), delete.version().get()),
        |_| list_encounters(),
    );

    view! {
        <h1>"Encounters"</h1>

        <ActionForm action=create attr:class="card form-row">
            <input type="text" name="name" placeholder="Encounter name" required />
            <button type="submit" class="primary">"New encounter"</button>
        </ActionForm>

        <Suspense fallback=move || view! { <p>"Loading…"</p> }>
            {move || {
                encounters
                    .get()
                    .map(|result| match result {
                        Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                        Ok(list) if list.is_empty() => {
                            view! { <p class="muted">"No encounters yet."</p> }.into_any()
                        }
                        Ok(list) => {
                            view! {
                                <div class="grid">
                                    {list
                                        .into_iter()
                                        .map(|e| {
                                            let id = e.id.clone();
                                            let href = format!("/combat/{}", e.id);
                                            view! {
                                                <section class="card">
                                                    <div class="card-head">
                                                        <A href=href>
                                                            <strong>{e.name.clone()}</strong>
                                                        </A>
                                                        <span class=format!("badge {}", e.status)>
                                                            {e.status.clone()}
                                                        </span>
                                                    </div>
                                                    <p class="muted">"Round " {e.round}</p>
                                                    <button
                                                        class="chip danger"
                                                        on:click=move |_| {
                                                            delete.dispatch(DeleteEncounter { id: id.clone() });
                                                        }
                                                    >
                                                        "Delete"
                                                    </button>
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
    }
}

#[component]
pub fn EncounterTrackerPage() -> impl IntoView {
    let params = use_params_map();
    let encounter_id = move || params.read().get("id").unwrap_or_default();

    let add_combatant = ServerAction::<AddCombatant>::new();
    let add_party = ServerAction::<AddPartyToEncounter>::new();
    let remove = ServerAction::<RemoveCombatant>::new();
    let roll_init = ServerAction::<RollAllInitiative>::new();
    let set_init = ServerAction::<SetInitiative>::new();
    let adjust_hp = ServerAction::<AdjustCombatantHp>::new();
    let toggle_condition = ServerAction::<ToggleCondition>::new();
    let toggle_conc = ServerAction::<ToggleConcentration>::new();
    let next_turn = ServerAction::<NextTurn>::new();
    let prev_turn = ServerAction::<PreviousTurn>::new();
    let end = ServerAction::<EndEncounter>::new();

    // Everything that can change the board invalidates both resources.
    let deps = move || {
        (
            encounter_id(),
            add_combatant.version().get(),
            add_party.version().get(),
            remove.version().get(),
            roll_init.version().get(),
            set_init.version().get(),
            adjust_hp.version().get(),
            toggle_condition.version().get(),
            toggle_conc.version().get(),
            next_turn.version().get(),
            prev_turn.version().get(),
            end.version().get(),
        )
    };

    let encounter = Resource::new(deps, |(id, ..)| get_encounter(id));
    let combatants = Resource::new(deps, |(id, ..)| list_combatants(id));

    view! {
        <Suspense fallback=move || view! { <p>"Loading encounter…"</p> }>
            {move || {
                encounter
                    .get()
                    .map(|result| match result {
                        Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                        Ok(None) => view! { <p class="error">"No such encounter."</p> }.into_any(),
                        Ok(Some(e)) => {
                            let (roll_id, next_id) = (e.id.clone(), e.id.clone());
                            let (prev_id, end_id) = (e.id.clone(), e.id.clone());
                            let party_id = e.id.clone();
                            view! {
                                <header class="sheet-head">
                                    <div>
                                        <h1>{e.name.clone()}</h1>
                                        <p class="muted">
                                            "Round " {e.round} " · " {e.status.clone()}
                                        </p>
                                    </div>
                                    <div class="row wrap">
                                        <button
                                            class="chip"
                                            on:click=move |_| {
                                                add_party
                                                    .dispatch(AddPartyToEncounter {
                                                        encounter_id: party_id.clone(),
                                                    });
                                            }
                                        >
                                            "Add party"
                                        </button>
                                        <button
                                            class="chip"
                                            on:click=move |_| {
                                                roll_init
                                                    .dispatch(RollAllInitiative {
                                                        encounter_id: roll_id.clone(),
                                                    });
                                            }
                                        >
                                            "Roll initiative"
                                        </button>
                                        <button
                                            class="chip"
                                            on:click=move |_| {
                                                prev_turn
                                                    .dispatch(PreviousTurn { encounter_id: prev_id.clone() });
                                            }
                                        >
                                            "◀ Prev"
                                        </button>
                                        <button
                                            class="primary"
                                            on:click=move |_| {
                                                next_turn
                                                    .dispatch(NextTurn { encounter_id: next_id.clone() });
                                            }
                                        >
                                            "Next turn ▶"
                                        </button>
                                        <button
                                            class="chip danger"
                                            on:click=move |_| {
                                                end.dispatch(EndEncounter { encounter_id: end_id.clone() });
                                            }
                                        >
                                            "End"
                                        </button>
                                    </div>
                                </header>

                                <ActionForm action=add_combatant attr:class="card form-row">
                                    <input type="hidden" name="encounter_id" value=e.id.clone() />
                                    <input type="text" name="name" placeholder="Monster" required />
                                    <input
                                        type="number"
                                        name="initiative"
                                        value="0"
                                        title="Initiative"
                                    />
                                    <input
                                        type="number"
                                        name="hp_max"
                                        value="10"
                                        min="1"
                                        title="Max HP"
                                    />
                                    <input
                                        type="number"
                                        name="armor_class"
                                        value="12"
                                        title="Armour class"
                                    />
                                    <input
                                        type="number"
                                        name="dex_score"
                                        value="10"
                                        title="Dexterity (tiebreak)"
                                    />
                                    <button type="submit">"Add"</button>
                                </ActionForm>

                                <TurnOrder
                                    combatants=combatants
                                    turn_index=e.turn_index
                                    adjust_hp=adjust_hp
                                    remove=remove
                                    set_init=set_init
                                    toggle_condition=toggle_condition
                                    toggle_conc=toggle_conc
                                />
                            }
                                .into_any()
                        }
                    })
            }}
        </Suspense>
    }
}

#[component]
fn TurnOrder(
    combatants: Resource<Result<Vec<crate::models::Combatant>, ServerFnError>>,
    turn_index: i64,
    adjust_hp: ServerAction<AdjustCombatantHp>,
    remove: ServerAction<RemoveCombatant>,
    set_init: ServerAction<SetInitiative>,
    toggle_condition: ServerAction<ToggleCondition>,
    toggle_conc: ServerAction<ToggleConcentration>,
) -> impl IntoView {
    view! {
        <Suspense fallback=move || view! { <p>"Loading combatants…"</p> }>
            {move || {
                combatants
                    .get()
                    .map(|result| match result {
                        Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                        Ok(list) if list.is_empty() => {
                            view! {
                                <p class="muted">"Nobody in the fight yet. Add the party or a monster."</p>
                            }
                                .into_any()
                        }
                        Ok(list) => {
                            view! {
                                <ol class="turn-order">
                                    {list
                                        .into_iter()
                                        .enumerate()
                                        .map(|(idx, c)| {
                                            let active = idx as i64 == turn_index;
                                            let (dmg_id, heal_id) = (c.id.clone(), c.id.clone());
                                            let (rm_id, init_id) = (c.id.clone(), c.id.clone());
                                            let conc_id = c.id.clone();
                                            let conditions = c.condition_list();
                                            let mut classes = String::from("card combatant");
                                            if active {
                                                classes.push_str(" active");
                                            }
                                            if c.is_down() {
                                                classes.push_str(" down");
                                            }
                                            if c.is_pc > 0 {
                                                classes.push_str(" pc");
                                            }
                                            view! {
                                                <li class=classes>
                                                    <div class="card-head">
                                                        <span class="init-badge">{c.initiative}</span>
                                                        <strong>{c.name.clone()}</strong>
                                                        {(c.is_pc > 0)
                                                            .then(|| view! { <span class="badge">"PC"</span> })}
                                                        {(c.concentrating > 0)
                                                            .then(|| {
                                                                view! { <span class="badge gold">"Concentrating"</span> }
                                                            })}
                                                        <span class="muted">"AC " {c.armor_class}</span>
                                                    </div>

                                                    <HpBar current=c.hp_current max=c.hp_max temp=c.hp_temp />

                                                    <div class="row wrap">
                                                        {[-10i64, -5, -1]
                                                            .into_iter()
                                                            .map(|delta| {
                                                                let id = dmg_id.clone();
                                                                view! {
                                                                    <button
                                                                        class="chip danger"
                                                                        on:click=move |_| {
                                                                            adjust_hp
                                                                                .dispatch(AdjustCombatantHp { id: id.clone(), delta });
                                                                        }
                                                                    >
                                                                        {delta.to_string()}
                                                                    </button>
                                                                }
                                                            })
                                                            .collect::<Vec<_>>()}
                                                        {[1i64, 5, 10]
                                                            .into_iter()
                                                            .map(|delta| {
                                                                let id = heal_id.clone();
                                                                view! {
                                                                    <button
                                                                        class="chip good"
                                                                        on:click=move |_| {
                                                                            adjust_hp
                                                                                .dispatch(AdjustCombatantHp { id: id.clone(), delta });
                                                                        }
                                                                    >
                                                                        {format!("+{delta}")}
                                                                    </button>
                                                                }
                                                            })
                                                            .collect::<Vec<_>>()}
                                                        <button
                                                            class={if c.concentrating > 0 { "chip gold" } else { "chip" }}
                                                            on:click=move |_| {
                                                                toggle_conc
                                                                    .dispatch(ToggleConcentration { id: conc_id.clone() });
                                                            }
                                                        >
                                                            "Conc."
                                                        </button>
                                                        <input
                                                            type="number"
                                                            class="init-input"
                                                            prop:value=c.initiative
                                                            title="Set initiative"
                                                            on:change=move |ev| {
                                                                if let Ok(v) = event_target_value(&ev).parse::<i64>() {
                                                                    set_init
                                                                        .dispatch(SetInitiative {
                                                                            id: init_id.clone(),
                                                                            initiative: v,
                                                                        });
                                                                }
                                                            }
                                                        />
                                                        <button
                                                            class="chip danger"
                                                            on:click=move |_| {
                                                                remove.dispatch(RemoveCombatant { id: rm_id.clone() });
                                                            }
                                                        >
                                                            "Remove"
                                                        </button>
                                                    </div>

                                                    <div class="row wrap conditions">
                                                        {CONDITIONS
                                                            .into_iter()
                                                            .map(|cond| {
                                                                let id = c.id.clone();
                                                                let on = conditions.iter().any(|x| x == cond);
                                                                view! {
                                                                    <button
                                                                        class={if on { "cond on" } else { "cond" }}
                                                                        on:click=move |_| {
                                                                            toggle_condition
                                                                                .dispatch(ToggleCondition {
                                                                                    id: id.clone(),
                                                                                    condition: cond.to_string(),
                                                                                });
                                                                        }
                                                                    >
                                                                        {cond}
                                                                    </button>
                                                                }
                                                            })
                                                            .collect::<Vec<_>>()}
                                                    </div>
                                                </li>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </ol>
                            }
                                .into_any()
                        }
                    })
            }}
        </Suspense>
    }
}
