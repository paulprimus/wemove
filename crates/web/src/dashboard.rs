
use topcoat::router::{page};
use topcoat::view::{view, View};
use topcoat::Result;


#[page("/dashboard")]
pub async fn dashboard() -> Result<impl View> {
    Ok(view! {
        <main class="mx-auto max-w-6xl px-6 py-16">
            <div class="rounded-2xl border border-white/10 bg-white/[0.04] p-8 sm:p-12">
                <p class="text-sm font-semibold uppercase tracking-[0.3em] text-cyan-400">"Workspace"</p>
                <h1 class="mt-3 text-4xl font-semibold tracking-tight text-white">"Willkommen im Dashboard"</h1>
                <p class="mt-4 max-w-xl text-lg text-slate-400">"Deine Kanban-Projekte werden hier erscheinen."</p>
            </div>

            <div class="mt-6" style="display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); column-gap: 1.5rem; row-gap: 1.5rem;">
                <div class="w-1/2 rounded-2xl border border-dashed border-cyan-400/40 bg-cyan-400/[0.04] p-6 transition-colors hover:border-cyan-400/70 hover:bg-cyan-400/[0.08]" style="width: 50%;">
                    <div class="flex h-12 w-12 items-center justify-center rounded-xl bg-cyan-400/10 text-2xl text-cyan-400">"+"</div>
                    <h2 class="mt-5 text-xl font-semibold text-white">"Neues Board erstellen"</h2>
                    <p class="mt-2 text-sm leading-6 text-slate-400">"Erstelle ein neues Board für dein nächstes Projekt."</p>
                </div>
                <div class="w-1/2 rounded-2xl border border-dashed border-cyan-400/40 bg-cyan-400/[0.04] p-6 transition-colors hover:border-cyan-400/70 hover:bg-cyan-400/[0.08]" style="width: 50%;">
                    <div class="flex h-12 w-12 items-center justify-center rounded-xl bg-cyan-400/10 text-2xl text-cyan-400">"≡"</div>
                    <h2 class="mt-5 text-xl font-semibold text-white">"Manage Boards"</h2>
                    <p class="mt-2 text-sm leading-6 text-slate-400">"Verwalte deine bestehenden Boards."</p>
                </div>
            </div>
        </main>
    })
}
