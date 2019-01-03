use std::collections::HashMap;
use std::error::Error;

use rand::{distributions::Alphanumeric, Rng};
use sha3::{Digest, Sha3_256};

use crate::pb;
use crate::persistence::{disk, gpg};
use crate::util::GenericError;

pub(crate) fn add(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let path = args.value_of("path").unwrap();
  let mut attributes: HashMap<String, pb::Attribute> = HashMap::new();

  for attribute in args.values_of("attributes").unwrap() {
    let attribute: Vec<&str> = attribute.splitn(2, '=').collect();
    if attribute.len() < 2 {
      return Err(GenericError::throw("could not parse attributes"));
    }

    let (key, value) = (&attribute[0], &attribute[1]);
    let mut attribute = pb::Attribute {
      value: value.to_string(),
      ..pb::Attribute::default()
    };

    if value == &"-" {
      attribute.value = random_secret();
      attribute.confidential = true;
    }
    if value == &"" {
      attribute.value = prompt_for_secret(key)?;
      attribute.confidential = true;
    }

    attributes.insert(key.to_string(), attribute);
  }

  let entry = pb::Entry {
    attributes: attributes,
    ..pb::Entry::default()
  };

  let mut vault = disk::get_vault()?;
  let destination = hash_path(path);

  disk::create_directories(&destination)?;
  disk::write_pack(&destination, &gpg::encrypt(disk::pack(&entry)?)?)?;

  disk::add_index(&mut vault, path, &destination);
  disk::write_metadata(&gpg::encrypt(disk::pack(&vault)?)?)?;

  Ok(())
}

pub(crate) fn edit(_args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  Ok(())
}

fn random_secret() -> String {
  rand::thread_rng()
    .sample_iter(&Alphanumeric)
    .take(16)
    .collect::<String>()
}

fn prompt_for_secret(key: &str) -> Result<String, Box<dyn Error>> {
  use colored::*;

  let secret = rpassword::prompt_password_stdout(&format!("Enter value for '{}': ", key.bold()))?;

  Ok(secret)
}

fn hash_path(path: &str) -> String {
  let mut hasher = Sha3_256::new();
  hasher.input(path);

  let hash = format!("{:x}", hasher.result());

  format!("{}/{}", &hash[0..2], hash)
}
