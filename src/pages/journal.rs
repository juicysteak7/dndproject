use leptos::prelude::*;

use crate::models::QUEST_STATUSES;
use crate::server::journal::{
    list_quests, list_sessions, AwardXp, CreateQuest, CreateSession, DeleteQuest, DeleteSession,
    SetQuestStatus,
};

#[component]
pub fn JournalPage() -> impl IntoView {
    let create_quest = ServerAction::<CreateQuest>::new();
    let set_status = ServerAction::<SetQuestStatus>::new();
    let delete_quest = ServerAction::<DeleteQuest>::new();

    let create_session = ServerAction::<CreateSession>::new();
    let delete_session = ServerAction::<DeleteSession>::new();
    let award_xp = ServerAction::<AwardXp>::new();

    let quests = Resource::new(
        move || {
            (
                create_quest.version().get(),
                set_status.version().get(),
                delete_quest.version().get(),
            )
        },
        |_| list_quests(),
    );
    let sessions = Resource::new(
        move || (create_session.version().get(), delete_session.version().get()),
        |_| list_sessions(),
    );

    view! {
        <h1>"Journal"</h1>

        <section class="card">
            <h3>"Quests"</h3>
            <ActionForm action=create_quest attr:class="form-row wrap">
                <input type="text" name="title" placeholder="Quest title" required />
                <input type="text" name="description" placeholder="Description" />
                <input type="text" name="reward" placeholder="Reward" />
                <select name="status">
                    {QUEST_STATUSES
                        .into_iter()
                        .map(|s| view! { <option value=s>{s}</option> })
                        .collect::<Vec<_>>()}
                </select>
                <button type="submit" class="primary">"Add quest"</button>
            </ActionForm>

            <Suspense fallback=move || view! { <p>"Loading…"</p> }>
                {move || {
                    quests
                        .get()
                        .map(|result| match result {
                            Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                            Ok(list) if list.is_empty() => {
                                view! { <p class="muted">"No quests recorded."</p> }.into_any()
                            }
                            Ok(list) => {
                                view! {
                                    <ul class="plain">
                                        {list
                                            .into_iter()
                                            .map(|q| {
                                                let (status_id, del_id) = (q.id.clone(), q.id.clone());
                                                let current = q.status.clone();
                                                view! {
                                                    <li class=format!("list-row quest {}", q.status)>
                                                        <div>
                                                            <strong>{q.title.clone()}</strong>
                                                            <span class=format!("badge {}", q.status)>
                                                                {q.status.clone()}
                                                            </span>
                                                            <p class="muted">{q.description.clone()}</p>
                                                            {(!q.reward.is_empty())
                                                                .then(|| {
                                                                    view! {
                                                                        <p class="muted">"Reward: " {q.reward.clone()}</p>
                                                                    }
                                                                })}
                                                        </div>
                                                        <div class="row wrap">
                                                            <select
                                                                prop:value=current
                                                                on:change=move |ev| {
                                                                    set_status
                                                                        .dispatch(SetQuestStatus {
                                                                            id: status_id.clone(),
                                                                            status: event_target_value(&ev),
                                                                        });
                                                                }
                                                            >
                                                                {QUEST_STATUSES
                                                                    .into_iter()
                                                                    .map(|s| view! { <option value=s>{s}</option> })
                                                                    .collect::<Vec<_>>()}
                                                            </select>
                                                            <button
                                                                class="chip danger"
                                                                on:click=move |_| {
                                                                    delete_quest.dispatch(DeleteQuest { id: del_id.clone() });
                                                                }
                                                            >
                                                                "×"
                                                            </button>
                                                        </div>
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
            <h3>"Award XP to the whole party"</h3>
            <div class="row wrap">
                {[50i64, 100, 250, 500, 1000]
                    .into_iter()
                    .map(|amount| {
                        view! {
                            <button
                                class="chip good"
                                on:click=move |_| {
                                    award_xp.dispatch(AwardXp { amount });
                                }
                            >
                                "+" {amount} " XP"
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
            {move || {
                award_xp
                    .value()
                    .get()
                    .map(|r| match r {
                        Ok(()) => view! { <p class="muted">"XP awarded."</p> }.into_any(),
                        Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                    })
            }}
        </section>

        <section class="card">
            <h3>"Session log"</h3>
            <ActionForm action=create_session attr:class="form-row wrap">
                <input type="text" name="title" placeholder="Session title" required />
                <input type="date" name="played_on" title="Date played" />
                <input type="text" name="summary" placeholder="One-line summary" />
                <input type="text" name="notes" placeholder="Notes" />
                <input type="number" name="xp_awarded" value="0" min="0" title="XP awarded" />
                <button type="submit" class="primary">"Log session"</button>
            </ActionForm>

            <Suspense fallback=move || view! { <p>"Loading…"</p> }>
                {move || {
                    sessions
                        .get()
                        .map(|result| match result {
                            Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                            Ok(list) if list.is_empty() => {
                                view! { <p class="muted">"No sessions logged yet."</p> }.into_any()
                            }
                            Ok(list) => {
                                view! {
                                    <ul class="plain">
                                        {list
                                            .into_iter()
                                            .map(|s| {
                                                let id = s.id.clone();
                                                view! {
                                                    <li class="list-row">
                                                        <div>
                                                            <strong>
                                                                "#" {s.session_number} " " {s.title.clone()}
                                                            </strong>
                                                            <span class="muted">" · " {s.played_on.clone()}</span>
                                                            <p>{s.summary.clone()}</p>
                                                            {(!s.notes.is_empty())
                                                                .then(|| {
                                                                    view! {
                                                                        <details>
                                                                            <summary>"Notes"</summary>
                                                                            <p>{s.notes.clone()}</p>
                                                                        </details>
                                                                    }
                                                                })}
                                                            {(s.xp_awarded > 0)
                                                                .then(|| {
                                                                    view! {
                                                                        <p class="muted">{s.xp_awarded} " XP awarded"</p>
                                                                    }
                                                                })}
                                                        </div>
                                                        <button
                                                            class="chip danger"
                                                            on:click=move |_| {
                                                                delete_session.dispatch(DeleteSession { id: id.clone() });
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
    }
}
