use std::error::Error;
use std::fs::File;
use std::path::Path;

use protobuf::parse_from_bytes;

use super::VaultContext;
use crate::gpg;
use crate::pb::*;
use crate::util;

impl Entry {
  pub fn read<P>(handle: &VaultContext, path: P) -> Result<Entry, Box<dyn Error>>
  where
    P: AsRef<Path>,
  {
    let pack = gpg::decrypt(&mut File::open(util::normalize_path(handle, &path))?)?;
    let message = parse_from_bytes::<Entry>(&pack)?;

    Ok(message)
  }

  pub fn add_attribute(&mut self, key: &str, value: &str) {
    let attribute = Attribute {
      value: value.to_string(),
      ..Attribute::default()
    };

    self.attributes.insert(key.to_string(), attribute);
  }

  pub fn add_confidential_attribute(&mut self, key: &str, value: &str) {
    let attribute = Attribute {
      value: value.to_string(),
      confidential: true,
      ..Attribute::default()
    };

    self.attributes.insert(key.to_string(), attribute);
  }

  pub fn add_file_attribute(&mut self, key: &str, value: &[u8]) {
    let mut attribute = Attribute {
      file: true,
      ..Attribute::default()
    };

    match String::from_utf8(value.to_vec()) {
      Ok(_) => attribute.bytes_value = value.to_vec(),
      Err(_) => attribute.bytes_value = value.to_vec(),
    }

    self.attributes.insert(key.to_string(), attribute);
  }
}

#[cfg(test)]
mod tests {
  use knox_testing::spec;

  use crate::prelude::*;

  #[test]
  fn read() {
    let tmp = spec::setup();
    let mut context = crate::spec::get_test_vault(tmp.path()).expect("could not get vault");

    let mut entry = Entry::default();
    entry.add_attribute("lorem", "ipsum");
    entry.add_attribute("foo", "bar");

    context
      .write_entry("pack.bin", &entry)
      .expect("could not write pack");

    let retrieved = context.read_entry("pack.bin").expect("could not read pack");

    assert_eq!(retrieved, entry);
  }
}
