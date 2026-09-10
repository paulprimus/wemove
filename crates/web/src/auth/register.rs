use std::future::Future;
use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{page, parse_query_params};
use topcoat::view::{View, view};

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
    Ok(view! {
        <main class="mx-auto flex min-h-[calc(100vh-73px)] max-w-6xl items-center justify-center px-6 py-16">
            <div class="w-full max-w-md rounded-2xl border border-border bg-surface p-8 shadow-sm">
                            <p class="text-sm font-semibold uppercase tracking-[0.3em] text-primary">"Get started"</p>
                            <h1 class="mt-3 text-3xl font-semibold tracking-tight text-foreground">"Create your account"</h1>
                            if let Some(message) = error_message {
                                <div class="mt-6 rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive" role="alert">(message)</div>
                            }
                            <form method="post" action="/auth/register" class="mt-8 space-y-5">
                                <div>
                                    <label for="name" class="mb-2 block text-sm font-medium text-foreground">"Name"</label>
                                    <input type="text" class="w-full rounded-lg border border-border bg-surface-strong px-4 py-3 text-foreground outline-none placeholder:text-muted-foreground focus:border-ring focus:ring-2 focus:ring-ring/20" id="name" name="name" required="" placeholder="Your name">
                                </div>
                                <div>
                                    <label for="email" class="mb-2 block text-sm font-medium text-foreground">"Email"</label>
                                    <input type="email" class="w-full rounded-lg border border-border bg-surface-strong px-4 py-3 text-foreground outline-none placeholder:text-muted-foreground focus:border-ring focus:ring-2 focus:ring-ring/20" id="email" name="email" required="" placeholder="you@example.com">
                                </div>
                                <div>
                                    <label for="password" class="mb-2 block text-sm font-medium text-foreground">"Password"</label>
                                    <input type="password" class="w-full rounded-lg border border-border bg-surface-strong px-4 py-3 text-foreground outline-none placeholder:text-muted-foreground focus:border-ring focus:ring-2 focus:ring-ring/20" id="password" name="password" required="" placeholder="Create a password">
                                </div>
                                <button type="submit" class="w-full rounded-lg bg-primary px-4 py-3 text-sm font-semibold text-primary-foreground hover:bg-primary/85 focus:outline-none focus:ring-2 focus:ring-ring">"Create account"</button>
                            </form>
                            <p class="mt-6 text-center text-sm text-muted-foreground">"Already have an account? " <a href="/login" class="font-medium text-primary hover:text-primary/80">"Sign in"</a></p>
            </div>
        </main>
    })
}
