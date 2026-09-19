use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::hp_bar::HpBar;
use crate::rules::format_bonus;
use crate::server::characters::{
    list_characters, AdjustHp, CreateCharacter, DeleteCharacter, LongRest,
};

#[component]
pub fn CharactersPage() -> impl IntoView {
    let create = ServerAction::<CreateCharacter>::new();
    let adjust_hp = ServerAction::<AdjustHp>::new();
    let long_rest = ServerAction::<LongRest>::new();
    let delete = ServerAction::<DeleteCharacter>::new();

    // Any mutation invalidates the roster.
    let characters = Resource::new(
        move || {
            (
                create.version().get(),
                adjust_hp.version().get(),
                long_rest.version().get(),
                delete.version().get(),
            )
        },
        |_| list_characters(),
    );

    view! {
        <h1>"Characters"</h1>

        <ActionForm action=create attr:class="card form-row">
            <input type="text" name="name" placeholder="Name" required />
            <input type="text" name="class" placeholder="Class" />
            <input type="text" name="race" placeholder="Race" />
            <input type="number" name="level" value="1" min="1" max="20" title="Level" />
            <button type="submit" class="primary">"Add character"</button>
        </ActionForm>

        <Suspense fallback=move || view! { <p>"Loading roster…"</p> }>
            {move || {
                characters
                    .get()
                    .map(|result| match result {
                        Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                        Ok(list) if list.is_empty() => {
                            view! { <p class="muted">"No characters yet."</p> }.into_any()
                        }
                        Ok(list) => {
                            view! {
                                <div class="grid">
                                    {list
                                        .into_iter()
                                        .map(|c| {
                                            let id = c.id.clone();
                                            let href = format!("/characters/{}", c.id);
                                            let (damage_id, heal_id) = (id.clone(), id.clone());
                                            let (rest_id, delete_id) = (id.clone(), id.clone());
                                            let subtitle = format!(
                                                "Level {} {} {}",
                                                c.level,
                                                c.race,
                                                c.class,
                                            );
                                            // Pull every field out before the view so the
                                            // closures below don't partially move `c`.
                                            let name = c.name.clone();
                                            let inspired = c.inspiration > 0;
                                            let (hp_current, hp_max, hp_temp) = (
                                                c.hp_current,
                                                c.hp_max,
                                                c.hp_temp,
                                            );
                                            let armor_class = c.armor_class;
                                            let speed = c.speed;
                                            let initiative = format_bonus(c.initiative_bonus());
                                            let passive_perception = c.passive_perception();
                                            view! {
                                                <section class="card character-card">
                                                    <div class="card-head">
                                                        <A href=href>
                                                            <strong>{name}</strong>
                                                        </A>
                                                        {inspired
                                                            .then(|| {
                                                                view! { <span class="badge">"Inspired"</span> }
                                                            })}
                                                    </div>
                                                    <p class="muted">{subtitle}</p>

                                                    <HpBar current=hp_current max=hp_max temp=hp_temp />

                                                    <p class="muted">
                                                        "AC " {armor_class} " · Init " {initiative} " · PP "
                                                        {passive_perception} " · Speed " {speed} "ft"
                                                    </p>

                                                    <div class="row wrap">
                                                        <button
                                                            class="chip danger"
                                                            on:click=move |_| {
                                                                adjust_hp
                                                                    .dispatch(AdjustHp {
                                                                        id: damage_id.clone(),
                                                                        delta: -5,
                                                                    });
                                                            }
                                                        >
                                                            "-5 HP"
                                                        </button>
                                                        <button
                                                            class="chip good"
                                                            on:click=move |_| {
                                                                adjust_hp
                                                                    .dispatch(AdjustHp {
                                                                        id: heal_id.clone(),
                                                                        delta: 5,
                                                                    });
                                                            }
                                                        >
                                                            "+5 HP"
                                                        </button>
                                                        <button
                                                            class="chip"
                                                            on:click=move |_| {
                                                                long_rest.dispatch(LongRest { id: rest_id.clone() });
                                                            }
                                                        >
                                                            "Long rest"
                                                        </button>
                                                        <button
                                                            class="chip danger"
                                                            on:click=move |_| {
                                                                delete
                                                                    .dispatch(DeleteCharacter {
                                                                        id: delete_id.clone(),
                                                                    });
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
    }
}
