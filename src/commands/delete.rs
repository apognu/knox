use std::error::Error;

use log::*;

use crate::vault::{pack, wire};

pub(crate) fn delete(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let mut vault = wire::get_vault()?;
  let path = args.value_of("path").unwrap();

  pack::delete(&mut vault, path)?;
  wire::remove_index(&mut vault, path);
  wire::write_metadata(&vault)?;

  info!("entry '{}' was successfully deleted from the vault", path);

  Ok(())
}
