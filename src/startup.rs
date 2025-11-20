// startup.rs

use std::env;

use clap::Parser;

use crate::*;

#[derive(Clone, Debug, Default, Parser)]
pub struct OptsCommon {
    #[arg(short, long)]
    pub verbose: bool,
    #[arg(short, long)]
    pub debug: bool,
    #[arg(short, long)]
    pub trace: bool,

    #[arg(long, default_value = "localhost")]
    pub mqtt_host: String,
    #[arg(long, default_value_t = 1883)]
    pub mqtt_port: u16,
    #[arg(long, default_value = "")]
    pub topic_prefix: String,
    #[arg(long, default_value = "test123")]
    pub topics: String,

    #[arg(long, default_value = "coap://localhost/store_data")]
    pub coap_url: String,
}

impl OptsCommon {
    pub fn finalize(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn get_loglevel(&self) -> Level {
        if self.trace {
            Level::TRACE
        } else if self.debug {
            Level::DEBUG
        } else if self.verbose {
            Level::INFO
        } else {
            Level::ERROR
        }
    }

    pub fn start_pgm(&self, name: &str) {
        tracing_subscriber::fmt()
            .with_max_level(self.get_loglevel())
            .with_target(false)
            .init();

        info!("Starting up {name} v{}...", env!("CARGO_PKG_VERSION"));
        debug!("Git branch: {}", env!("GIT_BRANCH"));
        debug!("Git commit: {}", env!("GIT_COMMIT"));
        debug!("Source timestamp: {}", env!("SOURCE_TIMESTAMP"));
        debug!("Compiler version: {}", env!("RUSTC_VERSION"));
    }
}
// EOF
