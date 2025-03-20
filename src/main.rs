use snafu::{ResultExt, Whatever};

use self::reader::{Reader, WealthOfNationsReader};

mod llm;
mod logger;
mod macros;
mod parser;
mod reader;

#[tokio::main]
#[snafu::report]
async fn main() -> Result<(), Whatever> {
    logger::init()?;
    let reader = WealthOfNationsReader::new();
    reader
        .summarize()
        .await
        .whatever_context("Failed to summarize")?;
    Ok(())
}
