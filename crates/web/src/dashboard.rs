use topcoat::context::Cx;
use topcoat::router::error::redirect;
use topcoat::router::page;
use topcoat::session;
use topcoat::view::view;
use topcoat::Result;

#[page("/dashboard")]
pub async fn dashboard(cx: &Cx) -> Result {
    if session::token_hash(cx).await?.is_none() {
        return Err(redirect("/login").into());
    }
    view! {
        <main class="container py-5">
            <div class="text-center">
                <h1 class="display-4 mb-3">"Willkommen im Dashboard"</h1>
                <p class="lead text-muted">"Deine Kanban-Projekte werden hier erscheinen"</p>
            </div>
        </main>
    }
}