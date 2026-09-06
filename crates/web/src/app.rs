use std::future::Future;
use topcoat::router::Slot;
use topcoat::view::View;
use topcoat::{context::Cx, router::layout, session, view::view, Result};

#[layout("/")]
pub async fn app_layout(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let is_authenticated = session::token_hash(cx).await?.is_some();
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>"WeMove"</title>
                <link rel="stylesheet" href=(topcoat::tailwind::stylesheet!())>
            </head>
            <body class="min-h-screen bg-slate-950 text-slate-100 antialiased">
                <header class="border-b border-white/10 bg-slate-950/90">
                    <nav class="mx-auto flex max-w-6xl items-center justify-between px-6 py-4" aria-label="Main navigation">
                        <a href="/" class="text-lg font-semibold tracking-tight text-white">"WeMove"</a>
                        if is_authenticated {
                            <details class="relative">
                                <summary class="cursor-pointer list-none rounded-lg border border-white/10 px-4 py-2 text-sm font-medium text-slate-200 hover:bg-white/10">"Menu"</summary>
                                <div class="absolute right-0 z-10 mt-2 w-44 rounded-xl border border-white/10 bg-slate-900 p-2 shadow-xl">
                                    <a href="/" class="block rounded-lg px-3 py-2 text-sm text-slate-300 hover:bg-white/10 hover:text-white">"Home"</a>
                                    <a href="/dashboard" class="block rounded-lg px-3 py-2 text-sm text-slate-300 hover:bg-white/10 hover:text-white">"Dashboard"</a>
                                    <form method="post" action="/auth/logout">
                                        <button type="submit" class="block w-full rounded-lg px-3 py-2 text-left text-sm text-slate-300 hover:bg-white/10 hover:text-white" style="display: block; width: 100%; text-align: left;">"Sign out"</button>
                                    </form>
                                </div>
                            </details>
                        }
                    </nav>
                </header>
                (slot)
            </body>
        </html>
    })
}
