use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::hp_bar::HpBar;
use crate::models::Character;
use crate::rules::{format_bonus, passive_score, Ability, SKILLS};
use crate::server::characters::{
    get_character, list_items, AddItem, AdjustHp, DeleteItem, LongRest, SetDeathSaves,
    ToggleEquipped, ToggleInspiration, ToggleSaveProficiency, ToggleSkillProficiency,
    UpdateCharacter,
};

#[component]
pub fn CharacterSheetPage() -> impl IntoView {
    let params = use_params_map();
    let character_id = move || params.read().get("id").unwrap_or_default();

    let update = ServerAction::<UpdateCharacter>::new();
    let adjust_hp = ServerAction::<AdjustHp>::new();
    let long_rest = ServerAction::<LongRest>::new();
    let toggle_skill = ServerAction::<ToggleSkillProficiency>::new();
    let toggle_save = ServerAction::<ToggleSaveProficiency>::new();
    let toggle_inspiration = ServerAction::<ToggleInspiration>::new();
    let death_saves = ServerAction::<SetDeathSaves>::new();

    let add_item = ServerAction::<AddItem>::new();
    let toggle_equipped = ServerAction::<ToggleEquipped>::new();
    let delete_item = ServerAction::<DeleteItem>::new();

    let character = Resource::new(
        move || {
            (
                character_id(),
                update.version().get(),
                adjust_hp.version().get(),
                long_rest.version().get(),
                toggle_skill.version().get(),
                toggle_save.version().get(),
                toggle_inspiration.version().get(),
                death_saves.version().get(),
            )
        },
        |(id, ..)| get_character(id),
    );

    let items = Resource::new(
        move || {
            (
                character_id(),
                add_item.version().get(),
                toggle_equipped.version().get(),
                delete_item.version().get(),
            )
        },
        |(id, ..)| list_items(id),
    );

    view! {
        <Suspense fallback=move || view! { <p>"Loading sheet…"</p> }>
            {move || {
                character
                    .get()
                    .map(|result| match result {
                        Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                        Ok(None) => view! { <p class="error">"No such character."</p> }.into_any(),
                        Ok(Some(c)) => {
                            view! {
                                <SheetHeader
                                    character=c.clone()
                                    adjust_hp=adjust_hp
                                    long_rest=long_rest
                                    toggle_inspiration=toggle_inspiration
                                    death_saves=death_saves
                                />
                                <div class="grid">
                                    <AbilityPanel character=c.clone() toggle_save=toggle_save />
                                    <SkillPanel character=c.clone() toggle_skill=toggle_skill />
                                </div>
                                <InventoryPanel
                                    character_id=c.id.clone()
                                    items=items
                                    add_item=add_item
                                    toggle_equipped=toggle_equipped
                                    delete_item=delete_item
                                />
                                <EditPanel character=c.clone() update=update />
                            }
                                .into_any()
                        }
                    })
            }}
        </Suspense>
    }
}

#[component]
fn SheetHeader(
    character: Character,
    adjust_hp: ServerAction<AdjustHp>,
    long_rest: ServerAction<LongRest>,
    toggle_inspiration: ServerAction<ToggleInspiration>,
    death_saves: ServerAction<SetDeathSaves>,
) -> impl IntoView {
    let c = character;
    let id = c.id.clone();
    let subtitle = format!(
        "Level {} {} {} {}",
        c.level,
        c.race,
        c.class,
        if c.subclass.is_empty() {
            String::new()
        } else {
            format!("({})", c.subclass)
        }
    );

    let hp_buttons = [-10i64, -5, -1, 1, 5, 10];
    let rest_id = id.clone();
    let insp_id = id.clone();
    let unconscious = c.is_unconscious();
    let (successes, failures) = (c.death_save_successes, c.death_save_failures);
    let (succ_id, fail_id) = (id.clone(), id.clone());

    view! {
        <header class="sheet-head">
            <div>
                <h1>{c.name.clone()}</h1>
                <p class="muted">
                    {subtitle}
                    {(!c.player_name.is_empty()).then(|| format!(" · played by {}", c.player_name))}
                </p>
            </div>
            <div class="stat-row">
                <Stat label="AC" value=c.armor_class.to_string() />
                <Stat label="Init" value=format_bonus(c.initiative_bonus()) />
                <Stat label="Speed" value=format!("{}ft", c.speed) />
                <Stat label="Prof" value=format_bonus(c.proficiency_bonus()) />
                <Stat label="Passive Perc" value=c.passive_perception().to_string() />
                <Stat label="XP" value=c.xp.to_string() />
            </div>
        </header>

        <section class="card">
            <div class="row wrap between">
                <HpBar current=c.hp_current max=c.hp_max temp=c.hp_temp />
                <div class="row wrap">
                    {hp_buttons
                        .into_iter()
                        .map(|delta| {
                            let btn_id = id.clone();
                            let class = if delta < 0 { "chip danger" } else { "chip good" };
                            view! {
                                <button
                                    class=class
                                    on:click=move |_| {
                                        adjust_hp.dispatch(AdjustHp { id: btn_id.clone(), delta });
                                    }
                                >
                                    {format_bonus(delta)}
                                </button>
                            }
                        })
                        .collect::<Vec<_>>()}
                    <button
                        class="chip"
                        on:click=move |_| {
                            long_rest.dispatch(LongRest { id: rest_id.clone() });
                        }
                    >
                        "Long rest"
                    </button>
                    <button
                        class={if c.inspiration > 0 { "chip good" } else { "chip" }}
                        on:click=move |_| {
                            toggle_inspiration
                                .dispatch(ToggleInspiration { id: insp_id.clone() });
                        }
                    >
                        "Inspiration"
                    </button>
                </div>
            </div>

            {unconscious
                .then(|| {
                    view! {
                        <div class="death-saves">
                            <strong class="danger-text">"Dying — death saves"</strong>
                            <div class="row">
                                <span>"Successes"</span>
                                {(1..=3i64)
                                    .map(|n| {
                                        let id = succ_id.clone();
                                        let filled = successes >= n;
                                        view! {
                                            <button
                                                class={if filled { "pip filled good" } else { "pip" }}
                                                on:click=move |_| {
                                                    // Clicking a filled pip clears back to it.
                                                    let next = if successes >= n { n - 1 } else { n };
                                                    death_saves
                                                        .dispatch(SetDeathSaves {
                                                            id: id.clone(),
                                                            successes: next,
                                                            failures,
                                                        });
                                                }
                                            ></button>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>
                            <div class="row">
                                <span>"Failures"</span>
                                {(1..=3i64)
                                    .map(|n| {
                                        let id = fail_id.clone();
                                        let filled = failures >= n;
                                        view! {
                                            <button
                                                class={if filled { "pip filled danger" } else { "pip" }}
                                                on:click=move |_| {
                                                    let next = if failures >= n { n - 1 } else { n };
                                                    death_saves
                                                        .dispatch(SetDeathSaves {
                                                            id: id.clone(),
                                                            successes,
                                                            failures: next,
                                                        });
                                                }
                                            ></button>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>
                        </div>
                    }
                })}
        </section>
    }
}

#[component]
fn Stat(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="stat">
            <span class="stat-value">{value}</span>
            <span class="stat-label">{label}</span>
        </div>
    }
}

#[component]
fn AbilityPanel(
    character: Character,
    toggle_save: ServerAction<ToggleSaveProficiency>,
) -> impl IntoView {
    let c = character;

    view! {
        <section class="card">
            <h3>"Abilities & saves"</h3>
            <table class="sheet-table">
                <thead>
                    <tr>
                        <th>"Ability"</th>
                        <th>"Score"</th>
                        <th>"Mod"</th>
                        <th>"Save"</th>
                    </tr>
                </thead>
                <tbody>
                    {Ability::ALL
                        .into_iter()
                        .map(|ability| {
                            let id = c.id.clone();
                            let proficient = c.is_save_proficient(ability);
                            view! {
                                <tr>
                                    <td>{ability.label()}</td>
                                    <td>{c.ability_score(ability)}</td>
                                    <td>{format_bonus(c.ability_modifier(ability))}</td>
                                    <td>
                                        <button
                                            class={if proficient { "pip filled" } else { "pip" }}
                                            title="Toggle save proficiency"
                                            on:click=move |_| {
                                                toggle_save
                                                    .dispatch(ToggleSaveProficiency {
                                                        id: id.clone(),
                                                        ability_key: ability.key().to_string(),
                                                    });
                                            }
                                        ></button>
                                        {format_bonus(c.save_bonus(ability))}
                                    </td>
                                </tr>
                            }
                        })
                        .collect::<Vec<_>>()}
                </tbody>
            </table>
        </section>
    }
}

#[component]
fn SkillPanel(
    character: Character,
    toggle_skill: ServerAction<ToggleSkillProficiency>,
) -> impl IntoView {
    let c = character;

    view! {
        <section class="card">
            <h3>"Skills"</h3>
            <p class="muted">"Click the left pip for proficiency, the right one for expertise."</p>
            <table class="sheet-table">
                <tbody>
                    {SKILLS
                        .into_iter()
                        .map(|(key, label, ability)| {
                            let (prof_id, exp_id) = (c.id.clone(), c.id.clone());
                            let (prof_key, exp_key) = (key.to_string(), key.to_string());
                            let proficient = c.is_skill_proficient(key);
                            let expert = c.has_expertise(key);
                            let bonus = c.skill_bonus(key).unwrap_or(0);
                            view! {
                                <tr>
                                    <td class="pips">
                                        <button
                                            class={if proficient { "pip filled" } else { "pip" }}
                                            title="Proficiency"
                                            on:click=move |_| {
                                                toggle_skill
                                                    .dispatch(ToggleSkillProficiency {
                                                        id: prof_id.clone(),
                                                        skill_key: prof_key.clone(),
                                                        expertise: false,
                                                    });
                                            }
                                        ></button>
                                        <button
                                            class={if expert { "pip filled gold" } else { "pip" }}
                                            title="Expertise"
                                            on:click=move |_| {
                                                toggle_skill
                                                    .dispatch(ToggleSkillProficiency {
                                                        id: exp_id.clone(),
                                                        skill_key: exp_key.clone(),
                                                        expertise: true,
                                                    });
                                            }
                                        ></button>
                                    </td>
                                    <td>{label}</td>
                                    <td class="muted">{ability.key().to_uppercase()}</td>
                                    <td class="mono">{format_bonus(bonus)}</td>
                                    <td class="muted">"pass " {passive_score(bonus)}</td>
                                </tr>
                            }
                        })
                        .collect::<Vec<_>>()}
                </tbody>
            </table>
        </section>
    }
}

#[component]
fn InventoryPanel(
    character_id: String,
    items: Resource<Result<Vec<crate::models::Item>, ServerFnError>>,
    add_item: ServerAction<AddItem>,
    toggle_equipped: ServerAction<ToggleEquipped>,
    delete_item: ServerAction<DeleteItem>,
) -> impl IntoView {
    view! {
        <section class="card">
            <h3>"Inventory"</h3>

            <ActionForm action=add_item attr:class="form-row">
                <input type="hidden" name="character_id" value=character_id />
                <input type="text" name="name" placeholder="Item" required />
                <input type="number" name="quantity" value="1" min="1" title="Quantity" />
                <input
                    type="number"
                    name="weight"
                    value="0"
                    step="0.1"
                    min="0"
                    title="Weight (lb)"
                />
                <input type="text" name="description" placeholder="Notes" />
                <button type="submit">"Add"</button>
            </ActionForm>

            <Suspense fallback=move || view! { <p>"Loading…"</p> }>
                {move || {
                    items
                        .get()
                        .map(|result| match result {
                            Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                            Ok(list) if list.is_empty() => {
                                view! { <p class="muted">"Carrying nothing."</p> }.into_any()
                            }
                            Ok(list) => {
                                let total_weight: f64 = list
                                    .iter()
                                    .map(|i| i.weight * i.quantity as f64)
                                    .sum();
                                view! {
                                    <p class="muted">
                                        {format!("{:.1} lb carried", total_weight)}
                                    </p>
                                    <table class="sheet-table">
                                        <tbody>
                                            {list
                                                .into_iter()
                                                .map(|item| {
                                                    let (eq_id, del_id) = (item.id.clone(), item.id.clone());
                                                    let equipped = item.equipped > 0;
                                                    view! {
                                                        <tr>
                                                            <td>
                                                                <button
                                                                    class={if equipped { "pip filled good" } else { "pip" }}
                                                                    title="Equipped"
                                                                    on:click=move |_| {
                                                                        toggle_equipped
                                                                            .dispatch(ToggleEquipped { item_id: eq_id.clone() });
                                                                    }
                                                                ></button>
                                                            </td>
                                                            <td>
                                                                {item.name.clone()}
                                                                {(item.quantity > 1)
                                                                    .then(|| format!(" ×{}", item.quantity))}
                                                            </td>
                                                            <td class="muted">{item.description.clone()}</td>
                                                            <td class="mono muted">
                                                                {format!("{:.1} lb", item.weight)}
                                                            </td>
                                                            <td>
                                                                <button
                                                                    class="chip danger"
                                                                    on:click=move |_| {
                                                                        delete_item
                                                                            .dispatch(DeleteItem { item_id: del_id.clone() });
                                                                    }
                                                                >
                                                                    "×"
                                                                </button>
                                                            </td>
                                                        </tr>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                }
                                    .into_any()
                            }
                        })
                }}
            </Suspense>
        </section>
    }
}

#[component]
fn EditPanel(character: Character, update: ServerAction<UpdateCharacter>) -> impl IntoView {
    let c = character;

    view! {
        <details class="card">
            <summary>"Edit sheet"</summary>
            <ActionForm action=update attr:class="edit-grid">
                <input type="hidden" name="id" value=c.id.clone() />

                <label>"Name" <input type="text" name="name" value=c.name.clone() required /></label>
                <label>
                    "Player" <input type="text" name="player_name" value=c.player_name.clone() />
                </label>
                <label>"Class" <input type="text" name="class" value=c.class.clone() /></label>
                <label>
                    "Subclass" <input type="text" name="subclass" value=c.subclass.clone() />
                </label>
                <label>"Race" <input type="text" name="race" value=c.race.clone() /></label>
                <label>
                    "Background" <input type="text" name="background" value=c.background.clone() />
                </label>
                <label>
                    "Alignment" <input type="text" name="alignment" value=c.alignment.clone() />
                </label>
                <label>
                    "Level"
                    <input type="number" name="level" value=c.level min="1" max="20" />
                </label>
                <label>"XP" <input type="number" name="xp" value=c.xp min="0" /></label>

                <label>"STR" <input type="number" name="str_score" value=c.str_score /></label>
                <label>"DEX" <input type="number" name="dex_score" value=c.dex_score /></label>
                <label>"CON" <input type="number" name="con_score" value=c.con_score /></label>
                <label>"INT" <input type="number" name="int_score" value=c.int_score /></label>
                <label>"WIS" <input type="number" name="wis_score" value=c.wis_score /></label>
                <label>"CHA" <input type="number" name="cha_score" value=c.cha_score /></label>

                <label>"Max HP" <input type="number" name="hp_max" value=c.hp_max min="1" /></label>
                <label>
                    "Current HP" <input type="number" name="hp_current" value=c.hp_current />
                </label>
                <label>
                    "Temp HP" <input type="number" name="hp_temp" value=c.hp_temp min="0" />
                </label>
                <label>"AC" <input type="number" name="armor_class" value=c.armor_class /></label>
                <label>"Speed" <input type="number" name="speed" value=c.speed /></label>
                <label>
                    "Hit dice" <input type="text" name="hit_dice" value=c.hit_dice.clone() />
                </label>

                <label class="wide">
                    "Notes" <textarea name="notes" rows="4">{c.notes.clone()}</textarea>
                </label>

                <button type="submit" class="primary">"Save sheet"</button>
            </ActionForm>
        </details>
    }
}
