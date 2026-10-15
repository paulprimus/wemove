use topcoat::Result;
use topcoat::view::{View, component, view};

pub struct BoardSummary {
    pub id: i64,
    pub name: String,
}

#[component]
pub async fn project_view(
    project_id: i64,
    name: &str,
    boards: Vec<BoardSummary>,
) -> Result<impl View> {
    Ok(view! {
        <main class="mx-auto max-w-6xl px-6 py-16">
            <a href="/dashboard" class="text-sm font-medium text-muted-foreground hover:text-foreground">"Back to dashboard"</a>
            <p class="mt-8 text-sm font-semibold uppercase tracking-[0.3em] text-primary">"Project"</p>
            <h1 class="mt-3 text-4xl font-semibold tracking-tight text-foreground">(name)</h1>
            <section class="mt-10">
                <div class="mb-5 flex items-center justify-between gap-4">
                    <h2 class="text-2xl font-semibold tracking-tight text-foreground">"Boards"</h2>
                    <a href=(format!("/projects/{project_id}/boards/new")) class="rounded-lg bg-primary px-4 py-2 text-sm font-semibold text-primary-foreground hover:bg-primary/85">"+ Create board"</a>
                </div>
                if boards.is_empty() {
                    <div class="rounded-2xl border border-dashed border-border-strong bg-surface p-8 text-center">
                        <p class="font-medium text-foreground">"No boards yet"</p>
                        <p class="mt-2 text-sm text-muted-foreground">"Create a board for this project to get started."</p>
                    </div>
                } else {
                    <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                        for board in boards {
                            <a href=(format!("/boards/{}", board.id)) class="block rounded-2xl border border-border bg-surface p-6 shadow-xs transition-colors hover:border-primary/60 hover:bg-surface-muted">
                                <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/10 text-xl text-primary">"≡"</div>
                                <h3 class="mt-4 text-lg font-semibold text-foreground">(board.name)</h3>
                                <p class="mt-1 text-sm text-muted-foreground">"Kanban board"</p>
                            </a>
                        }
                    </div>
                }
            </section>
        </main>
    })
}
