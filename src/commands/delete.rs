use std::error::Error;

use log::*;

use crate::prelude::*;

pub(crate) fn delete(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let mut vault = Vault::open()?;
  let path = args.value_of("path").unwrap();

  vault.delete_pack(path)?;
  vault.remove_index(path);
  vault.write()?;

  info!("entry '{}' was successfully deleted from the vault", path);

  Ok(())
}
