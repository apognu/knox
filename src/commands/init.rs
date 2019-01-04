use log::*;
use std::error::Error;

use crate::pb;
use crate::persistence::{disk, gpg};
use crate::util::GenericError;

pub(crate) fn init(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  if disk::pack_exists(disk::METADATA_FILE) {
    return Err(GenericError::throw(
      "a vault already exists, refusing to overwrite",
    ));
  }

  let identity = args.value_of("identity").unwrap();
  let vault = pb::Vault {
    identity: identity.to_string(),
    ..pb::Vault::default()
  };

  disk::write_metadata(&gpg::encrypt(disk::pack(&vault)?)?)?;

  info!("vault initialized successfully");

  Ok(())
}
