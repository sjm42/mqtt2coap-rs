// startup.rs

use log::*;
use std::env;
use structopt::StructOpt;

#[derive(Clone, Debug, Default, StructOpt)]
pub struct OptsCommon {
    #[structopt(short, long)]
    pub debug: bool,
    #[structopt(short, long)]
    pub trace: bool,
    #[structopt(long, default_value = "localhost")]
    pub mqtt_host: String,
    #[structopt(long, default_value = "1883")]
    pub mqtt_port: u16,
    #[structopt(long, default_value = "test123")]
    pub topics: String,
    #[structopt(long, default_value = "coap://localhost/store_data")]
    pub coap_url: String,
}

impl OptsCommon {
    pub fn finish(&mut self) -> anyhow::Result<()> {
        // self.blah = shellexpand::full(&self.blah)?.into_owned();

        Ok(())
    }
    fn get_loglevel(&self) -> LevelFilter {
        if self.trace {
            LevelFilter::Trace
        } else if self.debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        }
    }
}

pub fn start_pgm(c: &OptsCommon, desc: &str) {
    env_logger::Builder::new()
        .filter_level(c.get_loglevel())
        .format_timestamp_secs()
        .init();
    info!("Starting up {desc}...");
    info!("Git branch: {}", env!("GIT_BRANCH"));
    info!("Git commit: {}", env!("GIT_COMMIT"));
    info!("Source timestamp: {}", env!("SOURCE_TIMESTAMP"));
    info!("Compiler version: {}", env!("RUSTC_VERSION"));
}
// EOF
