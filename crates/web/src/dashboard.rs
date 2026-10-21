use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{error::redirect, page};
use topcoat::session;
use topcoat::runtime::link;
use topcoat::view::{View, attributes, component, view};

pub struct WorkspaceSummary {
    pub id: i64,
    pub name: String,
}

#[page("/dashboard")]
pub async fn dashboard(cx: &Cx) -> Result<impl View> {
    if session::token_hash(cx).await?.is_none() {
        return Err(redirect("/login").into());
    }

    Ok(view! { dashboard_view(workspaces: Vec::<WorkspaceSummary>::new()) })
}

#[component]
pub async fn dashboard_view(workspaces: Vec<WorkspaceSummary>) -> Result<impl View> {
    Ok(view! {
        <main class="mx-auto max-w-6xl px-6 py-16">
            <div class="rounded-2xl border border-border bg-surface p-8 shadow-sm sm:p-12">
                <p class="text-sm font-semibold uppercase tracking-[0.3em] text-primary">"Workspace"</p>
                <h1 class="mt-3 text-4xl font-semibold tracking-tight text-foreground">"Willkommen im Dashboard"</h1>
                <p class="mt-4 max-w-xl text-lg text-muted-foreground">"Deine Kanban-Projekte und nächsten Schritte auf einen Blick."</p>
            </div>

            <section class="mt-10">
                <div class="mb-5 flex items-center justify-between gap-4">
                    <h2 class="text-2xl font-semibold tracking-tight text-foreground">"Your workspaces"</h2>
                    <a href="/workspaces/new" class="rounded-lg bg-primary px-4 py-2 text-sm font-semibold text-primary-foreground hover:bg-primary/85 focus:outline-none focus:ring-2 focus:ring-ring">"+ Create workspace"</a>
                </div>
                if workspaces.is_empty() {
                    <div class="rounded-2xl border border-dashed border-border-strong bg-surface p-8 text-center">
                        <p class="font-medium text-foreground">"No workspaces yet"</p>
                        <p class="mt-2 text-sm text-muted-foreground">"Create a workspace to start organizing your work."</p>
                    </div>
                } else {
                    <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                        for workspace in workspaces {
                            link(
                                href: format!("/workspaces/{}", workspace.id),
                                attrs: attributes! { class="block rounded-2xl border border-border bg-surface p-6 shadow-xs transition-colors hover:border-primary/60 hover:bg-surface-muted" },
                                <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/10 text-xl text-primary">"≡"</div>
                                <h3 class="mt-4 text-lg font-semibold text-foreground">(workspace.name)</h3>
                                <p class="mt-1 text-sm text-muted-foreground">"Workspace with projects"</p>
                            )
                        }
                    </div>
                }
            </section>

            // <div class="mt-10 grid gap-6 sm:grid-cols-2">
            //     card(model: CardConfig {
            //         title: "Manage Boards",
            //         icon: "≡",
            //         description: "Verwalte deine bestehenden Boards.",
            //         href: Some("/boards")
            //     })
            // </div>
        </main>
    })
}
