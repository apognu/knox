use std::error::Error;

use log::*;

use crate::prelude::*;
use crate::util::{self, GenericError};

pub(crate) fn add(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let path = args.value_of("path").unwrap();
  let attributes = util::build_attributes(args)?;

  let entry = Entry {
    attributes,
    ..Entry::default()
  };

  let mut vault = Vault::open()?;
  let (salt, hash) = util::hash_path(path, None);

  util::create_parents(&hash)?;
  vault.write_pack(&path, &entry)?;

  vault.add_index(path, &salt);
  vault.write()?;

  info!("entry {} was successfully added to the vault", path);

  Ok(())
}

pub(crate) fn edit(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let path = args.value_of("path").unwrap();
  let attributes = util::build_attributes(args)?;

  let vault = Vault::open()?;

  if !vault.get_index().contains_key(path) {
    return Err(GenericError::throw("no entry was found at this path"));
  }

  let (_, real_path) = util::hash_path(path, Some(vault.get_index().get(path).unwrap()));
  let mut entry = Entry::read(&real_path)?;

  entry.mut_attributes().extend(attributes);
  vault.write_pack(&real_path, &entry)?;

  info!("entry {} was successfully edited", path);

  Ok(())
}
