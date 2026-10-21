use topcoat::Result;
use topcoat::runtime::link;
use topcoat::view::{View, attributes, component, view};

pub struct BoardSummary {
    pub id: i64,
    pub name: String,
}

pub struct ProjectMemberSummary {
    pub user_id: i64,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[component]
pub async fn project_view(
    project_id: i64,
    name: &str,
    boards: Vec<BoardSummary>,
    members: Vec<ProjectMemberSummary>,
) -> Result<impl View> {
    Ok(view! {
        <main class="mx-auto max-w-6xl px-6 py-16">
            link(
                href: "/dashboard",
                attrs: attributes! { class="text-sm font-medium text-muted-foreground hover:text-foreground" },
                "Back to dashboard"
            )
            <p class="mt-8 text-sm font-semibold uppercase tracking-[0.3em] text-primary">"Project"</p>
            <h1 class="mt-3 text-4xl font-semibold tracking-tight text-foreground">(name)</h1>
            <section class="mt-10">
                <div class="mb-5 flex items-center justify-between gap-4">
                    <h2 class="text-2xl font-semibold tracking-tight text-foreground">"Boards"</h2>
                    <a href=(format!("/projects/{project_id}/boards/new")) class="rounded-lg bg-primary px-4 py-2 text-sm font-semibold text-primary-foreground hover:bg-primary/85">"+ Create board"</a>
                </div>
                if boards.is_empty() {
                    <div class="rounded-2xl border border-dashed border-border-strong bg-surface p-8 text-center">
                        <p class="font-medium text-foreground">"No boards yet"</p>
                        <p class="mt-2 text-sm text-muted-foreground">"Create a board for this project to get started."</p>
                    </div>
                } else {
                    <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                        for board in boards {
                            link(
                                href: format!("/boards/{}", board.id),
                                attrs: attributes! { class="block rounded-2xl border border-border bg-surface p-6 shadow-xs transition-colors hover:border-primary/60 hover:bg-surface-muted" },
                                <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/10 text-xl text-primary">"≡"</div>
                                <h3 class="mt-4 text-lg font-semibold text-foreground">(board.name)</h3>
                                <p class="mt-1 text-sm text-muted-foreground">"Kanban board"</p>
                            )
                        }
                    </div>
                }
            </section>

            <section class="mt-24">
                <div class="mb-5 flex items-center justify-between gap-4">
                    <div>
                        <h2 class="text-2xl font-semibold tracking-tight text-foreground">"Project participants"</h2>
                        <p class="mt-1 text-sm text-muted-foreground">"Owners and admins are listed here."</p>
                    </div>
                    <a href=(format!("/projects/{project_id}/members")) class="rounded-lg border border-border px-4 py-2 text-sm font-medium text-foreground hover:bg-surface-muted">"Manage participants"</a>
                </div>
                <div class="rounded-2xl border border-border bg-surface">
                    for member in members {
                        if member.role == "owner" || member.role == "admin" {
                            <div class="flex items-center justify-between gap-4 p-5">
                                <div>
                                    <p class="font-medium text-foreground">(member.name.as_str())</p>
                                    <p class="text-sm text-muted-foreground">(member.email.as_str())</p>
                                </div>
                                if member.role == "owner" {
                                    <span class="rounded-full bg-primary/10 px-3 py-1 text-sm font-medium text-primary">"Owner"</span>
                                } else {
                                    <span class="rounded-full bg-surface-muted px-3 py-1 text-sm font-medium text-muted-foreground">"Admin"</span>
                                }
                            </div>
                        }
                    }
                </div>
            </section>
        </main>
    })
}

#[component]
pub async fn project_members_view(
    project_id: i64,
    name: &str,
    members: Vec<ProjectMemberSummary>,
    can_manage_members: bool,
    member_error: Option<String>,
) -> Result<impl View> {
    let member_error_message = member_error.map(|error| match error.as_str() {
        "invalid_email" => "Enter the email address of an existing user.",
        "user_not_found" => "No user with that email address was found.",
        "already_member" => "That user is already a project participant.",
        "invalid_role" => "Choose a valid project role.",
        _ => "The participant change could not be saved.",
    });

    Ok(view! {
        <main class="mx-auto max-w-4xl px-6 py-16">
            link(
                href: format!("/projects/{project_id}"),
                attrs: attributes! { class="text-sm font-medium text-muted-foreground hover:text-foreground" },
                "Back to project"
            )
            <p class="mt-8 text-sm font-semibold uppercase tracking-[0.3em] text-primary">"Project settings"</p>
            <h1 class="mt-3 text-4xl font-semibold tracking-tight text-foreground">"Participants"</h1>
            <p class="mt-2 text-muted-foreground">"Manage access to " (name) "."</p>
            if let Some(message) = member_error_message {
                <div class="mt-6 rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive" role="alert">(message)</div>
            }
            if can_manage_members {
                <form method="post" action=(format!("/projects/{project_id}/members/add")) class="mt-8 flex flex-col gap-3 rounded-2xl border border-border bg-surface p-5 sm:flex-row">
                    <input type="email" name="email" required="" placeholder="colleague@example.com" aria-label="Participant email" class="min-w-0 flex-1 rounded-lg border border-border bg-surface-strong px-4 py-2 text-foreground outline-none placeholder:text-muted-foreground focus:border-ring focus:ring-2 focus:ring-ring/20">
                    <select name="role" aria-label="Participant role" class="rounded-lg border border-border bg-surface-strong px-3 py-2 text-foreground">
                        <option value="member">"Member"</option>
                        <option value="admin">"Admin"</option>
                        <option value="viewer">"Viewer"</option>
                    </select>
                    <button type="submit" class="rounded-lg bg-primary px-4 py-2 text-sm font-semibold text-primary-foreground hover:bg-primary/85">"Add participant"</button>
                </form>
            }
            <div class="mt-6 divide-y divide-border rounded-2xl border border-border bg-surface">
                for member in members {
                    <div class="flex flex-col gap-4 p-5 sm:flex-row sm:items-center sm:justify-between">
                        <div class="min-w-0">
                            <p class="font-medium text-foreground">(member.name.as_str())</p>
                            <p class="text-sm text-muted-foreground">(member.email.as_str())</p>
                        </div>
                        if member.role == "owner" {
                            <span class="rounded-full bg-primary/10 px-3 py-1 text-sm font-medium text-primary">"Owner"</span>
                        } else if can_manage_members {
                            <div class="flex flex-wrap items-center gap-2">
                                <form method="post" action=(format!("/projects/{project_id}/members/update")) class="flex items-center gap-2">
                                    <input type="hidden" name="user_id" value=(member.user_id)>
                                    <select name="role" aria-label=(format!("Role for {}", member.name)) class="rounded-lg border border-border bg-surface-strong px-3 py-2 text-sm text-foreground">
                                        <option value="admin" selected=(member.role == "admin")>"Admin"</option>
                                        <option value="member" selected=(member.role == "member")>"Member"</option>
                                        <option value="viewer" selected=(member.role == "viewer")>"Viewer"</option>
                                    </select>
                                    <button type="submit" class="rounded-lg border border-border px-3 py-2 text-sm font-medium text-foreground hover:bg-surface-muted">"Save"</button>
                                </form>
                                <form method="post" action=(format!("/projects/{project_id}/members/remove"))>
                                    <input type="hidden" name="user_id" value=(member.user_id)>
                                    <button type="submit" class="rounded-lg px-3 py-2 text-sm font-medium text-destructive hover:bg-destructive/10">"Remove"</button>
                                </form>
                            </div>
                        } else {
                            <span class="rounded-full bg-surface-muted px-3 py-1 text-sm font-medium text-muted-foreground">(member.role.as_str())</span>
                        }
                    </div>
                }
            </div>
        </main>
    })
}
