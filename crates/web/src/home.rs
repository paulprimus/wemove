use std::future::Future;
use topcoat::context::Cx;
use topcoat::view::View;
use topcoat::{Result, router::page, session, view::view};

#[page("/")]
pub async fn index(cx: &Cx) -> Result<impl View> {
    let is_authenticated = session::token_hash(cx).await?.is_some();
    Ok(view! {
        <main class="mx-auto max-w-6xl px-6 py-20 sm:py-28">
            <div class="max-w-2xl">
                <p class="mb-5 text-sm font-semibold uppercase tracking-[0.3em] text-primary">"Project momentum"</p>
                <h1 class="text-5xl font-semibold tracking-tight text-foreground sm:text-7xl">"Move work forward."</h1>
                <p class="mt-6 max-w-xl text-lg leading-8 text-muted-foreground">"WeMove brings projects, people, and the next step into one focused workspace."</p>
                if is_authenticated {
                    <div class="mt-10 flex flex-wrap gap-4">
                        <a href="/logout" class="rounded-lg bg-primary px-5 py-3 text-sm font-semibold text-primary-foreground hover:bg-primary/85 focus:outline-none focus:ring-2 focus:ring-ring">"Sign out"</a>

                    </div>
                } else {
                    <div class="mt-10 flex flex-wrap gap-4">
                        <a href="/login" class="rounded-lg bg-primary px-5 py-3 text-sm font-semibold text-primary-foreground hover:bg-primary/85 focus:outline-none focus:ring-2 focus:ring-ring">"Sign in"</a>
                        <a href="/register" class="rounded-lg border border-border-strong px-5 py-3 text-sm font-semibold text-foreground hover:bg-surface-muted focus:outline-none focus:ring-2 focus:ring-ring">"Create an account"</a>
                    </div>
                }
            </div>
        </main>
    })
}
