use snafu::{ResultExt, Whatever};

use self::reader::{Reader, WealthOfNationsReader};

mod reader;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let reader = WealthOfNationsReader::new();
    let books = reader.read().whatever_context("Failed to read books")?;

    println!("{:?}", books);
    Ok(())
}
