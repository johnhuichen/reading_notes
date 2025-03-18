use snafu::Whatever;

use self::reader::{Reader, WealthOfNationsReader};

mod llm;
mod parser;
mod reader;

#[tokio::main]
#[snafu::report]
async fn main() -> Result<(), Whatever> {
    // let parser = WealthOfNationsParser::new();
    // let books = parser
    //     .parse()
    //     .whatever_context("Failed to parse wealth of nations")?;
    //
    // println!("{:?}", books);

    let reader = WealthOfNationsReader::new();
    let notes = reader.summarize().await;
    Ok(())
}
