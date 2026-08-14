use topcoat::router::page;
use topcoat::view::view;
use topcoat::Result;

#[page("/dashboard")]
pub async fn dashboard() -> Result {
    view! {
        <main class="container py-5">
            <div class="text-center">
                <h1 class="display-4 mb-3">"Willkommen im Dashboard"</h1>
                <p class="lead text-muted">"Deine Kanban-Projekte werden hier erscheinen"</p>
            </div>
        </main>
    }
}