use topcoat::Result;
use topcoat::runtime::link;
use topcoat::view::{View, attributes, component, view};

pub struct ProjectSummary {
    pub id: i64,
    pub name: String,
}

#[component]
pub async fn workspace_view(
    id: i64,
    name: &str,
    projects: Vec<ProjectSummary>,
) -> Result<impl View> {
    Ok(view! {
        <main class="mx-auto max-w-6xl px-6 py-16">
            link(
                href: "/dashboard",
                attrs: attributes! { class="text-sm font-medium text-muted-foreground hover:text-foreground" },
                "Back to workspaces"
            )
            <p class="mt-8 text-sm font-semibold uppercase tracking-[0.3em] text-primary">"Workspace"</p>
            <h1 class="mt-3 text-4xl font-semibold tracking-tight text-foreground">(name)</h1>
            <section class="mt-10">
                <div class="mb-5 flex items-center justify-between gap-4">
                    <h2 class="text-2xl font-semibold tracking-tight text-foreground">"Projects"</h2>
                    <a href=(format!("/workspaces/{id}/projects/new")) class="rounded-lg bg-primary px-4 py-2 text-sm font-semibold text-primary-foreground hover:bg-primary/85">"+ Create project"</a>
                </div>
                if projects.is_empty() {
                    <div class="rounded-2xl border border-dashed border-border-strong bg-surface p-8 text-center">
                        <p class="font-medium text-foreground">"No projects yet"</p>
                        <p class="mt-2 text-sm text-muted-foreground">"Create a project to start organizing your work."</p>
                    </div>
                } else {
                    <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                        for project in projects {
                            link(
                                href: format!("/projects/{}", project.id),
                                attrs: attributes! { class="block rounded-2xl border border-border bg-surface p-6 shadow-xs transition-colors hover:border-primary/60 hover:bg-surface-muted" },
                                <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/10 text-xl text-primary">"◈"</div>
                                <h3 class="mt-4 text-lg font-semibold text-foreground">(project.name)</h3>
                                <p class="mt-1 text-sm text-muted-foreground">"Project with Kanban boards"</p>
                            )
                        }
                    </div>
                }
            </section>
        </main>
    })
}
