use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{error::redirect, page};
use topcoat::session;
use topcoat::view::{View, component, view};

use crate::components::card::{CardConfig, card};

#[page("/dashboard")]
pub async fn dashboard(cx: &Cx) -> Result<impl View> {
    if session::token_hash(cx).await?.is_none() {
        return Err(redirect("/login").into());
    }

    Ok(view! { dashboard_view(boards: Vec::<String>::new()) })
}

#[component]
pub async fn dashboard_view(boards: Vec<String>) -> Result<impl View> {
    Ok(view! {
        <main class="mx-auto max-w-6xl px-6 py-16">
            <div class="rounded-2xl border border-border bg-surface p-8 shadow-sm sm:p-12">
                <p class="text-sm font-semibold uppercase tracking-[0.3em] text-primary">"Workspace"</p>
                <h1 class="mt-3 text-4xl font-semibold tracking-tight text-foreground">"Willkommen im Dashboard"</h1>
                <p class="mt-4 max-w-xl text-lg text-muted-foreground">"Deine Kanban-Projekte und nächsten Schritte auf einen Blick."</p>
            </div>

            <section class="mt-10">
                <div class="mb-5 flex items-center justify-between gap-4">
                    <h2 class="text-2xl font-semibold tracking-tight text-foreground">"Your boards"</h2>
                    <a href="/boards/new" class="rounded-lg bg-primary px-4 py-2 text-sm font-semibold text-primary-foreground hover:bg-primary/85 focus:outline-none focus:ring-2 focus:ring-ring">"+ Create board"</a>
                </div>
                if boards.is_empty() {
                    <div class="rounded-2xl border border-dashed border-border-strong bg-surface p-8 text-center">
                        <p class="font-medium text-foreground">"No boards yet"</p>
                        <p class="mt-2 text-sm text-muted-foreground">"Create a board to start organizing your work."</p>
                    </div>
                } else {
                    <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                        for board in boards {
                            <article class="rounded-2xl border border-border bg-surface p-6 shadow-xs">
                                <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/10 text-xl text-primary">"≡"</div>
                                <h3 class="mt-4 text-lg font-semibold text-foreground">(board)</h3>
                                <p class="mt-1 text-sm text-muted-foreground">"Kanban board"</p>
                            </article>
                        }
                    </div>
                }
            </section>

            <div class="mt-10 grid gap-6 sm:grid-cols-2">
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
