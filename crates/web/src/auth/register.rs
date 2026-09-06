use std::future::Future;
use topcoat::context::Cx;
use topcoat::router::{page, parse_query_params};
use topcoat::view::{view, View};
use topcoat::Result;

#[derive(serde::Deserialize)]
struct RegisterQuery {
    error: Option<String>,
}

#[page("/register")]
pub async fn register(cx: &Cx) -> Result<impl View> {
    let error_message = parse_query_params::<RegisterQuery>(cx)
        .ok()
        .and_then(|query| query.error)
        .map(|error| match error.as_str() {
            "missing_fields" => "Please fill in all fields.",
            "email_exists" => "This email address is already registered.",
            _ => "Registration failed. Please try again.",
        });
    Ok(
    view! {
        <main class="mx-auto flex min-h-[calc(100vh-73px)] max-w-6xl items-center justify-center px-6 py-16">
            <div class="w-full max-w-md rounded-2xl border border-white/10 bg-white/[0.04] p-8 shadow-2xl shadow-black/20">
                            <p class="text-sm font-semibold uppercase tracking-[0.3em] text-cyan-400">"Get started"</p>
                            <h1 class="mt-3 text-3xl font-semibold tracking-tight text-white">"Create your account"</h1>
                            if let Some(message) = error_message {
                                <div class="mt-6 rounded-lg border border-red-400/30 bg-red-400/10 px-4 py-3 text-sm text-red-200" role="alert">(message)</div>
                            }
                            <form method="post" action="/auth/register" class="mt-8 space-y-5">
                                <div>
                                    <label for="name" class="mb-2 block text-sm font-medium text-slate-200">"Name"</label>
                                    <input type="text" class="w-full rounded-lg border border-white/10 bg-slate-900 px-4 py-3 text-white outline-none placeholder:text-slate-600 focus:border-cyan-400 focus:ring-2 focus:ring-cyan-400/20" id="name" name="name" required="" placeholder="Your name">
                                </div>
                                <div>
                                    <label for="email" class="mb-2 block text-sm font-medium text-slate-200">"Email"</label>
                                    <input type="email" class="w-full rounded-lg border border-white/10 bg-slate-900 px-4 py-3 text-white outline-none placeholder:text-slate-600 focus:border-cyan-400 focus:ring-2 focus:ring-cyan-400/20" id="email" name="email" required="" placeholder="you@example.com">
                                </div>
                                <div>
                                    <label for="password" class="mb-2 block text-sm font-medium text-slate-200">"Password"</label>
                                    <input type="password" class="w-full rounded-lg border border-white/10 bg-slate-900 px-4 py-3 text-white outline-none placeholder:text-slate-600 focus:border-cyan-400 focus:ring-2 focus:ring-cyan-400/20" id="password" name="password" required="" placeholder="Create a password">
                                </div>
                                <button type="submit" class="w-full rounded-lg bg-cyan-400 px-4 py-3 text-sm font-semibold text-slate-950 hover:bg-cyan-300">"Create account"</button>
                            </form>
                            <p class="mt-6 text-center text-sm text-slate-400">"Already have an account? " <a href="/login" class="font-medium text-cyan-400 hover:text-cyan-300">"Sign in"</a></p>
            </div>
        </main>
    })
}
