use std::error::Error;

use crate::persistence::disk;
use crate::util::hierarchy;

pub(crate) fn list(_args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let vault = disk::get_vault()?;
  let list = hierarchy::build(&vault);

  println!("🔒 Vault store:");
  hierarchy::print(&mut vec![], &list);

  Ok(())
}

pub(crate) fn show(_args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  Ok(())
}
