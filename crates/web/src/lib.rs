pub mod app;
pub mod auth;
pub mod components;
pub mod dashboard;
pub mod home;

use topcoat::router::RouterBuilder;

use crate::app::app_layout;
use crate::auth::login::login;
use crate::auth::register::register as register_page;
use crate::dashboard::dashboard as dashboard_page;
use crate::home::index;

pub fn register(builder: RouterBuilder) -> RouterBuilder {
    builder
        .layout(app_layout)
        .page(index)
        .page(login)
        .page(register_page)
        .page(dashboard_page)
}
