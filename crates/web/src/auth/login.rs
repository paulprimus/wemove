use topcoat::{router::page, view::view, Result};

#[page("/login")]
pub async fn login() -> Result {
    view! {
        <main class="container py-5">
            <div class="row justify-content-center">
                <div class="col-md-6 col-lg-4">
                    <div class="card shadow">
                        <div class="card-body p-4">
                            <h1 class="h3 mb-4 fw-normal text-center">"WeMove"</h1>
                            <form method="post" action="/auth/login">
                                <div class="mb-3">
                                    <label for="email" class="form-label">"Email"</label>
                                    <input type="email" class="form-control" id="email" name="email" required="" placeholder="you@example.com">
                                </div>
                                <div class="mb-3">
                                    <label for="password" class="form-label">"Password"</label>
                                    <input type="password" class="form-control" id="password" name="password" required="" placeholder="Enter your password">
                                </div>
                                <button type="submit" class="btn btn-primary w-100">"Sign In"</button>
                            </form>
                            <p class="text-center mt-3 text-muted">"Don't have an account? " <a href="/register">"Sign up"</a></p>
                        </div>
                    </div>
                </div>
            </div>
        </main>
    }
}