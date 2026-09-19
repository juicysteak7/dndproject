use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::components::{Route, Router, Routes, A};
use leptos_router::path;

use crate::pages::character_sheet::CharacterSheetPage;
use crate::pages::characters::CharactersPage;
use crate::pages::combat::{EncounterListPage, EncounterTrackerPage};
use crate::pages::journal::JournalPage;
use crate::pages::overview::OverviewPage;
use crate::pages::world::WorldPage;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/dnd-dashboard.css" />
        <Title text="D&D Dashboard" />
        <Router>
            <div class="layout">
                <nav class="sidebar">
                    <span class="brand">"⚔ D&D Dashboard"</span>
                    <A href="/">"Overview"</A>
                    <A href="/characters">"Characters"</A>
                    <A href="/combat">"Combat"</A>
                    <A href="/world">"World"</A>
                    <A href="/journal">"Journal"</A>
                </nav>
                <main>
                    <Routes fallback=|| {
                        view! { <p class="error">"Page not found."</p> }
                    }>
                        <Route path=path!("/") view=OverviewPage />
                        <Route path=path!("/characters") view=CharactersPage />
                        <Route path=path!("/characters/:id") view=CharacterSheetPage />
                        <Route path=path!("/combat") view=EncounterListPage />
                        <Route path=path!("/combat/:id") view=EncounterTrackerPage />
                        <Route path=path!("/world") view=WorldPage />
                        <Route path=path!("/journal") view=JournalPage />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}
