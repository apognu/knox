use std::error::Error;

use log::*;

use crate::persistence::disk;

pub(crate) fn delete(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let mut vault = disk::get_vault()?;
  let path = args.value_of("path").unwrap();

  disk::delete_pack(&mut vault, path)?;

  info!("entry '{}' was successfully deleted from the vault", path);

  Ok(())
}
