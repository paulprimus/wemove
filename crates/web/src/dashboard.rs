use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{error::redirect, page};
use topcoat::session;
use topcoat::view::{View, view};

use crate::components::card::{CardConfig, card};

#[page("/dashboard")]
pub async fn dashboard(cx: &Cx) -> Result<impl View> {
    if session::token_hash(cx).await?.is_none() {
        return Err(redirect("/login").into());
    }

    Ok(view! {
        <main class="mx-auto max-w-6xl px-6 py-16">
            <div class="rounded-2xl border border-border bg-surface p-8 shadow-sm sm:p-12">
                <p class="text-sm font-semibold uppercase tracking-[0.3em] text-primary">"Workspace"</p>
                <h1 class="mt-3 text-4xl font-semibold tracking-tight text-foreground">"Willkommen im Dashboard"</h1>
                <p class="mt-4 max-w-xl text-lg text-muted-foreground">"Deine Kanban-Projekte werden hier erscheinen."</p>
            </div>

            <div class="mt-6 grid gap-6 sm:grid-cols-2">
                card(model: CardConfig {
                    title: "Neues Board erstellen",
                    icon: "+",
                    description: "Erstelle ein neues Board für dein nächstes Projekt.",
                    href: Some("/boards/new")
                })
                card(model: CardConfig {
                    title: "Manage Boards",
                    icon: "≡",
                    description: "Verwalte deine bestehenden Boards.",
                    href: Some("/boards")
                })
            </div>
        </main>
    })
}
