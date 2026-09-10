use topcoat::Result;
use topcoat::view::{View, component, view};

pub struct CardConfig<'a> {
    pub title: &'a str,
    pub icon: &'a str,
    pub description: &'a str,
    pub href: Option<&'a str>,
}

#[component]
pub async fn card<'a>(model: CardConfig<'a>) -> Result<impl View> {
    let CardConfig {
        title,
        icon,
        description,
        href,
    } = model;

    Ok(view! {
        if let Some(href) = href {
            <a href=(href) class="block rounded-2xl border border-dashed border-primary/40 bg-primary/5 p-6 no-underline transition-colors hover:border-primary/70 hover:bg-primary/10">
                <div class="flex h-12 w-12 items-center justify-center rounded-xl bg-primary/10 text-2xl text-primary">(icon)</div>
                <h2 class="mt-5 text-xl font-semibold text-foreground">(title)</h2>
                <p class="mt-2 text-sm leading-6 text-muted-foreground">(description)</p>
            </a>
        } else {
            <div class="rounded-2xl border border-dashed border-primary/40 bg-primary/5 p-6 transition-colors hover:border-primary/70 hover:bg-primary/10">
                <div class="flex h-12 w-12 items-center justify-center rounded-xl bg-primary/10 text-2xl text-primary">(icon)</div>
                <h2 class="mt-5 text-xl font-semibold text-foreground">(title)</h2>
                <p class="mt-2 text-sm leading-6 text-muted-foreground">(description)</p>
            </div>
        }
    })
}
