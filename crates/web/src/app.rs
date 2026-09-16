use common::{AuthState, CurrentUser};
use std::future::Future;
use topcoat::router::Slot;
use topcoat::view::View;
use topcoat::{
    Result,
    context::{Cx, try_app_context},
    router::layout,
    session,
    view::view,
};

#[layout("/")]
pub async fn app_layout(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let current_user = current_user(cx).await;
    let is_authenticated = current_user.is_some();
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>"WeMove"</title>
                topcoat::runtime::script()
                <link rel="stylesheet" href=(topcoat::tailwind::stylesheet!())>
            </head>
            <body class="min-h-screen bg-background text-foreground antialiased">
                <header class="border-b border-border bg-background/90">
                    <nav class="mx-auto flex max-w-6xl items-center justify-between px-6 py-4" aria-label="Main navigation">
                        <a href="/" class="text-lg font-semibold tracking-tight text-foreground">"WeMove"</a>
                        <div class="flex items-center gap-3">
                            <button
                                type="button"
                                class="inline-flex h-10 w-10 items-center justify-center rounded-lg border border-border text-muted-foreground transition-colors hover:bg-surface-muted hover:text-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                                aria-label="Toggle color theme"
                                title="Toggle color theme"
                                @click="document.documentElement.classList.toggle('light')"
                            >
                                <svg class="theme-icon-moon size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
                                    <path d="M20.5 15.2A8.5 8.5 0 0 1 8.8 3.5 8.5 8.5 0 1 0 20.5 15.2Z"></path>
                                </svg>
                                <svg class="theme-icon-sun size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
                                    <circle cx="12" cy="12" r="3.5"></circle>
                                    <path d="M12 2v2M12 20v2M4.93 4.93l1.42 1.42M17.65 17.65l1.42 1.42M2 12h2M20 12h2M4.93 19.07l1.42-1.42M17.65 6.35l1.42-1.42"></path>
                                </svg>
                            </button>
                        if is_authenticated {
                            if let Some(user) = &current_user {
                                <span class="hidden text-sm text-muted-foreground sm:inline">(user.name.as_str())</span>
                            }
                            <details class="relative">
                                <summary class="cursor-pointer list-none rounded-lg border border-border px-4 py-2 text-sm font-medium text-foreground hover:bg-surface-muted">"Menu"</summary>
                                <div class="absolute right-0 z-10 mt-2 w-44 rounded-xl border border-border bg-surface-strong p-2 shadow-sm">
                                    <a href="/" class="block rounded-lg px-3 py-2 text-sm text-muted-foreground hover:bg-surface-muted hover:text-foreground">"Home"</a>
                                    <a href="/dashboard" class="block rounded-lg px-3 py-2 text-sm text-muted-foreground hover:bg-surface-muted hover:text-foreground">"Dashboard"</a>
                                    <form method="post" action="/auth/logout">
                                        <button type="submit" class="block w-full rounded-lg px-3 py-2 text-left text-sm text-muted-foreground hover:bg-surface-muted hover:text-foreground">"Sign out"</button>
                                    </form>
                                </div>
                            </details>
                        } else {
                            <div class="flex items-center gap-2">
                                <a href="/login" class="rounded-lg px-3 py-2 text-sm font-medium text-muted-foreground hover:bg-surface-muted hover:text-foreground">"Sign in"</a>
                                <a href="/register" class="rounded-lg bg-primary px-3 py-2 text-sm font-semibold text-primary-foreground hover:bg-primary/85">"Create account"</a>
                            </div>
                        }
                        </div>
                    </nav>
                </header>
                (slot)
            </body>
        </html>
    })
}

async fn current_user(cx: &Cx) -> Option<CurrentUser> {
    let hash = session::token_hash(cx).await.ok()??;
    let auth_state = try_app_context::<AuthState>(cx)?;
    let conn = auth_state.db.connect().ok()?;
    let mut statement = conn
        .prepare(
            "SELECT users.name, users.email
             FROM sessions
             INNER JOIN users ON users.id = sessions.user_id
             WHERE sessions.token_hash = ?1 AND sessions.expires_at > ?2",
        )
        .await
        .ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    let mut rows = statement
        .query((
            hash.as_ref()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
            now,
        ))
        .await
        .ok()?;
    let row = rows.next().await.ok()??;
    let name = match row.get_value(0).ok()? {
        turso::Value::Text(value) => value,
        _ => return None,
    };
    let email = match row.get_value(1).ok()? {
        turso::Value::Text(value) => value,
        _ => return None,
    };

    Some(CurrentUser { name, email })
}
