use log::*;
use std::error::Error;

use crate::vault::wire;

pub(crate) fn init(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let identity = args.value_of("identity").unwrap();

  wire::write_metadata(&wire::create_metadata(identity)?)?;

  info!("vault initialized successfully");

  Ok(())
}
