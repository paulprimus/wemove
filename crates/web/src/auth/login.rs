use std::future::Future;
use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{page, parse_query_params};
use topcoat::view::{View, view};

#[derive(serde::Deserialize)]
struct LoginQuery {
    error: Option<String>,
}

#[page("/login")]
pub async fn login(cx: &Cx) -> Result<impl View> {
    let error_message = parse_query_params::<LoginQuery>(cx)
        .ok()
        .and_then(|query| query.error)
        .map(|error| match error.as_str() {
            "missing_fields" => "Please enter your email and password.",
            "invalid_credentials" => "Invalid email or password.",
            "session_error" => "Unable to start your session. Please try again.",
            _ => "Login failed. Please try again.",
        });
    Ok(view! {
        <main class="mx-auto flex min-h-[calc(100vh-73px)] max-w-6xl items-center justify-center px-6 py-16">
            <div class="w-full max-w-md rounded-2xl border border-border bg-surface p-8 shadow-sm">
                            <p class="text-sm font-semibold uppercase tracking-[0.3em] text-primary">"Welcome back"</p>
                            <h1 class="mt-3 text-3xl font-semibold tracking-tight text-foreground">"Sign in to WeMove"</h1>
                            if let Some(message) = error_message {
                                <div class="mt-6 rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive" role="alert">(message)</div>
                            }
                            <form method="post" action="/auth/login" class="mt-8 space-y-5">
                                <div>
                                    <label for="email" class="mb-2 block text-sm font-medium text-foreground">"Email"</label>
                                    <input type="email" class="w-full rounded-lg border border-border bg-surface-strong px-4 py-3 text-foreground outline-none placeholder:text-muted-foreground focus:border-ring focus:ring-2 focus:ring-ring/20" id="email" name="email" required="" placeholder="you@example.com">
                                </div>
                                <div>
                                    <label for="password" class="mb-2 block text-sm font-medium text-foreground">"Password"</label>
                                    <input type="password" class="w-full rounded-lg border border-border bg-surface-strong px-4 py-3 text-foreground outline-none placeholder:text-muted-foreground focus:border-ring focus:ring-2 focus:ring-ring/20" id="password" name="password" required="" placeholder="Enter your password">
                                </div>
                                <button type="submit" class="w-full rounded-lg bg-primary px-4 py-3 text-sm font-semibold text-primary-foreground hover:bg-primary/85 focus:outline-none focus:ring-2 focus:ring-ring">"Sign in"</button>
                            </form>
                            <p class="mt-6 text-center text-sm text-muted-foreground">"Don't have an account? " <a href="/register" class="font-medium text-primary hover:text-primary/80">"Sign up"</a></p>
            </div>
        </main>
    })
}
