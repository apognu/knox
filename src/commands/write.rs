use std::error::Error;

use log::*;

use crate::pb;
use crate::util::{self, GenericError};
use crate::vault::{pack, wire};

pub(crate) fn add(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let path = args.value_of("path").unwrap();
  let attributes = util::build_attributes(args)?;

  let entry = pb::Entry {
    attributes,
    ..pb::Entry::default()
  };

  let mut vault = wire::get_vault()?;
  let (salt, hash) = wire::hash_path(path, None);

  wire::create_parents(&hash)?;
  pack::write(&vault, &hash, &wire::pack(&entry)?)?;

  wire::add_index(&mut vault, path, &salt);
  wire::write_metadata(&vault)?;

  info!("entry {} was successfully added to the vault", path);

  Ok(())
}

pub(crate) fn edit(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let path = args.value_of("path").unwrap();
  let attributes = util::build_attributes(args)?;

  let vault = wire::get_vault()?;

  if !vault.get_index().contains_key(path) {
    return Err(GenericError::throw("no entry was found at this path"));
  }

  let (_, real_path) = wire::hash_path(path, Some(vault.get_index().get(path).unwrap()));
  let mut entry: pb::Entry = pack::read(&real_path)?;

  entry.mut_attributes().extend(attributes);
  pack::write(&vault, &real_path, &wire::pack(&entry)?)?;

  info!("entry {} was successfully edited", path);

  Ok(())
}
