use topcoat::Result;
use topcoat::runtime::link;
use topcoat::view::{View, attributes, component, view};

#[component]
pub async fn create_workspace_form(error: Option<String>) -> Result<impl View> {
    let error_message = error.map(|error| match error.as_str() {
        "empty_name" => "Enter a name for your workspace.",
        "name_too_long" => "Workspace names must be 100 characters or fewer.",
        _ => "We couldn't create your workspace. Please try again.",
    });
    Ok(view! {
        <main class="mx-auto flex min-h-[calc(100vh-73px)] max-w-6xl items-center justify-center px-6 py-16">
            <section class="w-full max-w-xl rounded-2xl border border-border bg-surface p-8 shadow-sm sm:p-10">
                link(href: "/dashboard", attrs: attributes! { class="text-sm font-medium text-muted-foreground hover:text-foreground" }, "Back to dashboard")
                <p class="mt-8 text-sm font-semibold uppercase tracking-[0.3em] text-primary">"Organization"</p>
                <h1 class="mt-3 text-3xl font-semibold tracking-tight text-foreground">"Create a workspace"</h1>
                <p class="mt-3 text-muted-foreground">"Workspaces contain projects, teams, and shared settings."</p>
                if let Some(message) = error_message {
                    <div class="mt-6 rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive" role="alert">(message)</div>
                }
                <form method="post" action="/workspaces/new" class="mt-8 space-y-5">
                    <label for="workspace-name" class="block text-sm font-medium text-foreground">"Workspace name"</label>
                    <input type="text" id="workspace-name" name="name" required="" autofocus="" maxlength="100" class="w-full rounded-lg border border-border bg-surface-strong px-4 py-3 text-foreground outline-none placeholder:text-muted-foreground focus:border-ring focus:ring-2 focus:ring-ring/20" placeholder="e.g. Acme Inc.">
                    <div class="flex justify-end gap-3">
                        <a href="/dashboard" class="rounded-lg px-4 py-3 text-sm font-medium text-muted-foreground hover:bg-surface-muted hover:text-foreground">"Cancel"</a>
                        <button type="submit" class="rounded-lg bg-primary px-5 py-3 text-sm font-semibold text-primary-foreground hover:bg-primary/85">"Create workspace"</button>
                    </div>
                </form>
            </section>
        </main>
    })
}
