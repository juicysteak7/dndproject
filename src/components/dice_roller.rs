use leptos::prelude::*;

use crate::server::rolls::RollDice;

/// Dice roller. Rolls happen server-side; the client only parses for feedback.
#[component]
pub fn DiceRoller() -> impl IntoView {
    let roll = ServerAction::<RollDice>::new();
    let (expression, set_expression) = signal("d20".to_string());

    // Live validation so a typo is obvious before you click.
    let parse_error = move || match crate::dice::parse(&expression.get()) {
        Ok(_) => None,
        Err(e) => Some(e.to_string()),
    };

    let dispatch_roll = move |expr: String| {
        if crate::dice::parse(&expr).is_ok() {
            roll.dispatch(RollDice { expression: expr });
        }
    };

    let presets = ["d20", "2d6", "1d8+3", "4d6", "d100"];

    view! {
        <section class="card dice-roller">
            <h3>"Dice"</h3>

            <div class="row">
                <input
                    type="text"
                    class="dice-input"
                    prop:value=move || expression.get()
                    on:input=move |ev| set_expression.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            dispatch_roll(expression.get_untracked());
                        }
                    }
                />
                <button
                    class="primary"
                    disabled=move || parse_error().is_some()
                    on:click=move |_| dispatch_roll(expression.get_untracked())
                >
                    "Roll"
                </button>
            </div>

            <div class="row wrap">
                {presets
                    .into_iter()
                    .map(|p| {
                        view! {
                            <button
                                class="chip"
                                on:click=move |_| {
                                    set_expression.set(p.to_string());
                                    dispatch_roll(p.to_string());
                                }
                            >
                                {p}
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>

            {move || parse_error().map(|e| view! { <p class="error">{e}</p> })}

            {move || {
                roll.value()
                    .get()
                    .map(|result| match result {
                        Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                        Ok(outcome) => {
                            let detail = outcome
                                .rolls
                                .iter()
                                .map(|r| r.to_string())
                                .collect::<Vec<_>>()
                                .join(" + ");
                            let crit_class = match outcome.critical {
                                Some(true) => "roll-total crit-success",
                                Some(false) => "roll-total crit-fail",
                                None => "roll-total",
                            };
                            view! {
                                <div class="roll-result">
                                    <span class=crit_class>{outcome.total}</span>
                                    <span class="roll-detail">
                                        {outcome.expression.clone()} " → [" {detail} "]"
                                        {(outcome.modifier != 0)
                                            .then(|| {
                                                crate::rules::format_bonus(outcome.modifier)
                                            })}
                                    </span>
                                    {match outcome.critical {
                                        Some(true) => Some(view! { <span class="crit">"Natural 20"</span> }),
                                        Some(false) => {
                                            Some(view! { <span class="crit">"Natural 1"</span> })
                                        }
                                        None => None,
                                    }}
                                </div>
                            }
                                .into_any()
                        }
                    })
            }}
        </section>
    }
}
