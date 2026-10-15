use topcoat::Result;
use topcoat::view::{View, component, view};

#[component]
pub async fn board_view(name: &str) -> Result<impl View> {
    Ok(view! {
        <main class="mx-auto max-w-6xl px-6 py-16">
            <a href="/dashboard" class="text-sm font-medium text-muted-foreground hover:text-foreground">
                "Back to dashboard"
            </a>
            <div class="mt-8 flex items-start justify-between gap-6">
                <div>
                    <p class="text-sm font-semibold uppercase tracking-[0.3em] text-primary">"Board"</p>
                    <h1 class="mt-3 text-4xl font-semibold tracking-tight text-foreground">(name)</h1>
                    <p class="mt-3 text-muted-foreground">"Your Kanban board is ready."</p>
                </div>
            </div>
            <section class="mt-10 rounded-2xl border border-dashed border-border-strong bg-surface p-8 text-center">
                <p class="font-medium text-foreground">"No cards yet"</p>
                <p class="mt-2 text-sm text-muted-foreground">"Columns and cards can be added here next."</p>
            </section>
        </main>
    })
}
