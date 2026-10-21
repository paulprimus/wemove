use topcoat::Result;
use topcoat::runtime::link;
use topcoat::view::{View, attributes, component, view};

#[component]
pub async fn create_project_form(workspace_id: i64, error: Option<String>) -> Result<impl View> {
    let error_message = error.map(|error| match error.as_str() {
        "empty_name" => "Enter a name for your project.",
        "name_too_long" => "Project names must be 100 characters or fewer.",
        _ => "We couldn't create your project. Please try again.",
    });
    Ok(view! {
        <main class="mx-auto flex min-h-[calc(100vh-73px)] max-w-6xl items-center justify-center px-6 py-16">
            <section class="w-full max-w-xl rounded-2xl border border-border bg-surface p-8 shadow-sm sm:p-10">
                link(href: "/dashboard", attrs: attributes! { class="text-sm font-medium text-muted-foreground hover:text-foreground" }, "Back to dashboard")
                <p class="mt-8 text-sm font-semibold uppercase tracking-[0.3em] text-primary">"Workspace"</p>
                <h1 class="mt-3 text-3xl font-semibold tracking-tight text-foreground">"Create a project"</h1>
                <p class="mt-3 text-muted-foreground">"Projects group the boards you use to organize your work."</p>
                if let Some(message) = error_message {
                    <div class="mt-6 rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive" role="alert">(message)</div>
                }
                <form method="post" action=(format!("/workspaces/{workspace_id}/projects")) class="mt-8 space-y-5">
                    <label for="project-name" class="block text-sm font-medium text-foreground">"Project name"</label>
                    <input type="text" id="project-name" name="name" required="" autofocus="" maxlength="100" class="w-full rounded-lg border border-border bg-surface-strong px-4 py-3 text-foreground outline-none placeholder:text-muted-foreground focus:border-ring focus:ring-2 focus:ring-ring/20" placeholder="e.g. Product launch">
                    <div class="flex justify-end gap-3">
                        <a href="/dashboard" class="rounded-lg px-4 py-3 text-sm font-medium text-muted-foreground hover:bg-surface-muted hover:text-foreground">"Cancel"</a>
                        <button type="submit" class="rounded-lg bg-primary px-5 py-3 text-sm font-semibold text-primary-foreground hover:bg-primary/85 focus:outline-none focus:ring-2 focus:ring-ring">"Create project"</button>
                    </div>
                </form>
            </section>
        </main>
    })
}
