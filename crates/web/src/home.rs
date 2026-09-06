use std::future::Future;
use topcoat::{router::page, view::view, Result};
use topcoat::view::View;

#[page("/")]
pub async fn index() -> Result<impl View> {
    Ok(view! {
        <main class="mx-auto max-w-6xl px-6 py-20 sm:py-28">
            <div class="max-w-2xl">
                <p class="mb-5 text-sm font-semibold uppercase tracking-[0.3em] text-cyan-400">"Project momentum"</p>
                <h1 class="text-5xl font-semibold tracking-tight text-white sm:text-7xl">"Move work forward."</h1>
                <p class="mt-6 max-w-xl text-lg leading-8 text-slate-400">"WeMove brings projects, people, and the next step into one focused workspace."</p>
                <div class="mt-10 flex flex-wrap gap-4">
                    <a href="/login" class="rounded-lg bg-cyan-400 px-5 py-3 text-sm font-semibold text-slate-950 hover:bg-cyan-300">"Sign in"</a>
                    <a href="/register" class="rounded-lg border border-white/15 px-5 py-3 text-sm font-semibold text-white hover:bg-white/10">"Create an account"</a>
                </div>
            </div>
        </main>
    })
}
