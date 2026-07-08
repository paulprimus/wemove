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