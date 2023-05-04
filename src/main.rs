use cda::runner::Runner;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    Runner::run()?;
    Ok(())
}
