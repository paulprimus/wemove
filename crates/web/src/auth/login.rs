use topcoat::context::Cx;
use topcoat::router::{page, uri};
use topcoat::view::view;
use topcoat::Result;

#[page("/login")]
pub async fn login(cx: &Cx) -> Result {
    let error_message = uri(cx).query().and_then(|query| {
        query.split('&').find_map(|parameter| {
            let (key, value) = parameter.split_once('=')?;
            if key != "error" {
                return None;
            }

            Some(match value {
                "missing_fields" => "Please enter your email and password.",
                "invalid_credentials" => "Invalid email or password.",
                "session_error" => "Unable to start your session. Please try again.",
                _ => "Login failed. Please try again.",
            })
        })
    });

    view! {
        <main class="container py-5">
            <div class="row justify-content-center">
                <div class="col-md-6 col-lg-4">
                    <div class="card shadow">
                        <div class="card-body p-4">
                            <h1 class="h3 mb-4 fw-normal text-center">"WeMove"</h1>
                            if let Some(message) = error_message {
                                <div class="alert alert-danger" role="alert">(message)</div>
                            }
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
