use topcoat::{
    router::{page, RouterBuilder},
    view::view,
    Result,
};

#[page("/")]
pub async fn home() -> Result {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>"WeMove"</title>
            </head>
            <body>
                <main>
                    <h1>"WeMove"</h1>
                    <p>"Welcome to the WeMove application."</p>
                </main>
            </body>
        </html>
    }
}

pub fn register(builder: RouterBuilder) -> RouterBuilder {
    builder.page(home)
}
