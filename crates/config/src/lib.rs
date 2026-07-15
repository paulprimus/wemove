use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[arg(long, env = "HOST", default_value = "127.0.0.1")]
    pub host: String,

    #[arg(long, env = "PORT", default_value = "8080")]
    pub port: u16,

    #[arg(long, env = "RUST_LOG", default_value = "info")]
    pub log_level: String,

    #[arg(long)]
    pub config: Option<PathBuf>,
}

impl Args {
    pub fn load() -> Self {
        dotenvy::dotenv().ok();
        Self::parse()
    }

    pub fn merge_with_config(&self) -> (SocketAddr, String) {
        let addr: SocketAddr = format!("{}:{}", self.host, self.port).parse().unwrap();
        (addr, self.log_level.clone())
    }
}

/// Authentifizierungs-Konfiguration, die ausschließlich über Umgebungsvariablen
/// (bzw. eine `.env`-Datei) gesetzt werden kann - bewusst kein CLI-Flag, damit
/// Secrets nicht über Prozessargumente/Shell-History/`--help` sichtbar werden.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_expiry_secs: u64,
}

impl AuthConfig {
    /// Lädt die Auth-Konfiguration aus der Umgebung (inkl. `.env`, falls vorhanden).
    ///
    /// # Panics
    /// Falls `JWT_SECRET` nicht gesetzt ist, da ein leerer/Default-Secret in
    /// Produktion ein Sicherheitsrisiko darstellt.
    pub fn load() -> Self {
        dotenvy::dotenv().ok();

        let jwt_secret = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET muss über die .env-Datei oder als Umgebungsvariable gesetzt werden");

        let token_expiry_secs = std::env::var("TOKEN_EXPIRY_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3600);

        Self {
            jwt_secret,
            token_expiry_secs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_default() {
        let args = Args::parse_from(["test"]);
        assert_eq!(args.host, "127.0.0.1");
        assert_eq!(args.port, 8080);
    }
}