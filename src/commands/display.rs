use std::error::Error;

use crate::pb;
use crate::persistence::disk;
use crate::util::{display, hierarchy, GenericError};

pub(crate) fn list(_args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let vault = disk::get_vault()?;
  let list = hierarchy::build(&vault);

  println!("🔒 Vault store:");
  hierarchy::print(&mut vec![], &list);

  Ok(())
}

pub(crate) fn show(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let vault = disk::get_vault()?;
  let path = args.value_of("path").unwrap();

  if !vault.get_index().contains_key(path) {
    return Err(GenericError::throw("no entry was found at this path"));
  }

  let entry: pb::Entry = disk::read_pack(vault.get_index().get(path).unwrap())?;

  display::entry(path, &entry);

  Ok(())
}
