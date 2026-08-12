use topcoat::{router::page, view::view, Result};

#[page("/")]
pub async fn index() -> Result {
    view! {
        <main class="container py-5">
            <div class="text-center">
                <h1 class="display-4 mb-3">"WeMove"</h1>
                <p class="lead text-muted mb-4">"we ship projects"</p>
                <div class="d-flex gap-2 justify-content-center">
                    <a href="/login" class="btn btn-primary">"Sign In"</a>
                    <a href="/register" class="btn btn-outline-secondary">"Sign Up"</a>
                </div>
            </div>
        </main>
    }
}