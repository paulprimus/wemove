#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            app_name: "wemove".to_string(),
        }
    }
}
