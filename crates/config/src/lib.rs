use clap::Parser;
use serde::Deserialize;
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

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
}

impl Config {
    pub fn load(config_path: Option<PathBuf>) -> anyhow::Result<Config> {
        let config_path = config_path
            .or_else(|| Some(PathBuf::from("config.toml")))
            .filter(|p| p.exists());

        let mut settings = config::Config::builder();

        if let Some(path) = config_path {
            settings = settings.add_source(config::File::from(path));
        }

        let config = settings.build()?.try_deserialize::<Config>()?;
        Ok(config)
    }
}

impl Args {
    pub fn load() -> Self {
        Self::parse()
    }

    pub fn merge_with_config(&self, config: Option<Config>) -> (SocketAddr, String) {
        let host = config
            .as_ref()
            .map(|c| c.server.host.clone())
            .unwrap_or_else(|| self.host.clone());

        let port = config
            .as_ref()
            .map(|c| c.server.port)
            .unwrap_or(self.port);

        let log_level = config
            .as_ref()
            .map(|c| c.logging.level.clone())
            .unwrap_or_else(|| self.log_level.clone());

        let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

        (addr, log_level)
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