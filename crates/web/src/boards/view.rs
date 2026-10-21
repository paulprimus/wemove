use topcoat::Result;
use topcoat::view::{View, component, view};

pub struct TicketSummary {
    pub title: String,
    pub description: String,
}

pub struct TicketColumn {
    pub title: String,
    pub tickets: Vec<TicketSummary>,
}

#[component]
pub async fn board_view(
    board_id: i64,
    name: &str,
    columns: Vec<TicketColumn>,
    can_edit: bool,
    error: Option<String>,
) -> Result<impl View> {
    let error_message = error.map(|error| match error.as_str() {
        "empty_title" => "Enter a title for your ticket.",
        "title_too_long" => "Ticket titles must be 200 characters or fewer.",
        "description_too_long" => "Ticket descriptions must be 5000 characters or fewer.",
        _ => "We couldn't create your ticket. Please try again.",
    });

    Ok(view! {
        <main class="mx-auto max-w-6xl px-6 py-16">
            <a href="/dashboard" class="text-sm font-medium text-muted-foreground hover:text-foreground">
                "Back to dashboard"
            </a>
            <div class="mt-8 flex items-start justify-between gap-6">
                <div>
                    <p class="text-sm font-semibold uppercase tracking-[0.3em] text-primary">"Board"</p>
                    <h1 class="mt-3 text-4xl font-semibold tracking-tight text-foreground">(name)</h1>
                </div>
            </div>
            if let Some(message) = error_message {
                <div class="mt-6 rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive" role="alert">(message)</div>
            }
            if can_edit {
                <form method="post" action=(format!("/boards/{board_id}/tickets")) class="mt-8 space-y-3 rounded-2xl border border-border bg-surface p-6">
                    <input
                        type="text"
                        name="title"
                        required=""
                        maxlength="200"
                        class="w-full rounded-lg border border-border bg-surface-strong px-4 py-3 text-foreground outline-none placeholder:text-muted-foreground focus:border-ring focus:ring-2 focus:ring-ring/20"
                        placeholder="Ticket title"
                    >
                    <textarea
                        name="description"
                        maxlength="5000"
                        rows="3"
                        class="w-full rounded-lg border border-border bg-surface-strong px-4 py-3 text-foreground outline-none placeholder:text-muted-foreground focus:border-ring focus:ring-2 focus:ring-ring/20"
                        placeholder="Description (optional)"
                    ></textarea>
                    <div class="flex justify-end">
                        <button type="submit" class="rounded-lg bg-primary px-5 py-3 text-sm font-semibold text-primary-foreground hover:bg-primary/85 focus:outline-none focus:ring-2 focus:ring-ring">"+ Create ticket"</button>
                    </div>
                </form>
            }
            <section class="mt-10 grid gap-4 md:grid-cols-3">
                for column in columns {
                    <div class="rounded-2xl border border-border bg-surface-muted p-4">
                        <h2 class="mb-3 text-sm font-semibold uppercase tracking-wider text-muted-foreground">(column.title)</h2>
                        <div class="space-y-3">
                            for ticket in column.tickets {
                                <div class="rounded-xl border border-border bg-surface p-4 shadow-xs">
                                    <p class="font-medium text-foreground">(ticket.title)</p>
                                    <p class="mt-1 whitespace-pre-line text-sm text-muted-foreground">(ticket.description)</p>
                                </div>
                            }
                        </div>
                    </div>
                }
            </section>
        </main>
    })
}
