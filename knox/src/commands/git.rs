use std::error::Error;

use libknox::*;
use log::*;

use crate::util::vault_path;

pub(crate) fn set_remote(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let vault = VaultContext::open(vault_path()?)?;
  let origin = args.value_of("url").unwrap();

  vault.set_git_origin(origin)?;

  info!("git remote URL set to '{}'", origin);

  Ok(())
}

pub(crate) fn push(_args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let vault = VaultContext::open(vault_path()?)?;

  vault.push()?;

  info!("vault modifications successfully pushed upstream");

  Ok(())
}
