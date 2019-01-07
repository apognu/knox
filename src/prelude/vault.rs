use std::error::Error;
use std::fs::{create_dir_all, remove_dir, remove_file, File, OpenOptions};
use std::io::Write;
use std::path::Path;

use protobuf::parse_from_bytes;

use super::Packing;
use crate::gpg;
use crate::pb::*;
use crate::util::{self, VaultError};

impl Vault {
  pub fn create(identity: &str) -> Result<Vault, Box<dyn Error>> {
    if Vault::has_pack(util::METADATA_FILE) {
      return Err(VaultError::throw(
        "a vault already exists, refusing to overwrite",
      ));
    }

    gpg::get_keys(&mut gpg::get_context()?, identity)?;

    let vault = Vault {
      identity: identity.to_string(),
      ..Vault::default()
    };

    Ok(vault)
  }

  pub fn open() -> Result<Vault, Box<dyn Error>> {
    if !Path::new(&util::normalize_path(&util::METADATA_FILE)).exists() {
      return Err(VaultError::throw(&format!(
        "vault does not exist at {}, please initialize it",
        util::normalize_path(&""),
      )));
    }

    let pack = gpg::decrypt(&mut File::open(util::normalize_path(&util::METADATA_FILE))?)?;
    let vault = parse_from_bytes::<Vault>(&pack)?;

    Ok(vault)
  }

  pub fn write(&self) -> Result<(), Box<dyn Error>> {
    create_dir_all(util::normalize_path(&""))?;

    let mut file = OpenOptions::new()
      .create(true)
      .truncate(true)
      .write(true)
      .open(util::normalize_path(&util::METADATA_FILE))?;

    file.write_all(&gpg::encrypt(self, &self.pack()?)?)?;

    Ok(())
  }

  pub fn add_index(&mut self, path: &str, destination: &str) {
    self
      .mut_index()
      .insert(path.to_string(), destination.to_string());
  }

  pub fn remove_index(&mut self, path: &str) {
    self.mut_index().remove(path);
  }

  pub fn read_entry(&self, path: &str) -> Result<Entry, Box<dyn Error>> {
    if !self.get_index().contains_key(path) {
      return Err(VaultError::throw("no entry was found at this path"));
    }

    let path = util::hash_path(self.get_index().get(path));
    let entry = Entry::read(path)?;

    Ok(entry)
  }

  pub fn write_entry(&mut self, path: &str, entry: &Entry) -> Result<(), Box<dyn Error>> {
    let hash = util::hash_path(self.get_index().get(path));

    util::create_parents(&hash)?;

    let mut file = OpenOptions::new()
      .create(true)
      .truncate(true)
      .write(true)
      .open(&util::normalize_path(&hash))?;

    file.write_all(&gpg::encrypt(self, &entry.pack()?)?)?;

    self.add_index(path, &hash);
    self.write()?;

    Ok(())
  }

  pub(crate) fn delete_entry(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
    if let Some(salt) = self.get_index().get(path) {
      let hash = util::hash_path(Some(salt));

      self.write()?;
      remove_file(util::normalize_path(&hash))?;

      for directory in Path::new(&hash).ancestors() {
        let _ = remove_dir(util::normalize_path(&format!("{}", directory.display())));
      }

      self.remove_index(&path);

      return Ok(());
    }

    Err(VaultError::throw(
      "requested entry does not exist in the vault",
    ))
  }

  pub fn has_pack<P>(path: P) -> bool
  where
    P: AsRef<Path>,
  {
    Path::new(&util::normalize_path(&path)).exists()
  }
}

#[cfg(test)]
mod tests {
  use crate::prelude::*;
  use crate::util;

  #[test]
  fn create() {
    let _tmp = crate::tests::setup();

    Vault::create(crate::tests::GPG_IDENTITY)
      .expect("could not create vault")
      .write()
      .expect("could not write metadata");

    let vault = Vault::open().expect("could not retrieve vault");

    assert_eq!(crate::tests::GPG_IDENTITY, vault.get_identity());
  }

  #[test]
  fn read_and_write() {
    let _tmp = crate::tests::setup();

    let vault = Vault {
      identity: crate::tests::GPG_IDENTITY.to_string(),
      ..Vault::default()
    };

    vault.write().expect("could not write pack");
    let retrieved: Vault = Vault::open().expect("could not read pack");

    assert_eq!(vault, retrieved);
  }

  #[test]
  fn add_index() {
    let _tmp = crate::tests::setup();
    let mut vault = crate::tests::get_test_vault();

    vault.write().expect("could not write metadata");
    vault.add_index("foo/bar", "lorem/ipsum");
    vault.write().expect("could not write metadata");

    let vault = Vault::open().expect("coult not get vault");

    assert_eq!(
      "lorem/ipsum",
      vault
        .get_index()
        .get("foo/bar")
        .expect("could not find index")
    );
  }

  #[test]
  fn remove_index() {
    let _tmp = crate::tests::setup();
    let mut vault = crate::tests::get_test_vault();

    vault
      .mut_index()
      .insert("foo/bar".to_string(), "lorem/ipsum".to_string());

    vault.write().expect("could not write metadata");
    vault.remove_index("foo/bar");
    vault.write().expect("could not write metadata");

    let vault = Vault::open().expect("could not retrieve metadata");

    assert_eq!(None, vault.get_index().get("foo/bar"));
  }

  #[test]
  fn pack() {
    let message = Vault { ..Vault::default() };

    let wired = message.pack().expect("could not create pack");
    let retrieved: Vault = super::parse_from_bytes(&wired).expect("could not read pack");

    assert_eq!(message, retrieved);
  }

  #[test]
  fn has_pack() {
    let _tmp = crate::tests::setup();

    std::fs::File::create(util::normalize_path(&"test")).expect("could not create file");

    assert_eq!(true, Vault::has_pack("test"));
    assert_eq!(false, Vault::has_pack("foobar"));
  }

  #[test]
  fn read_and_write_entry() {
    let _tmp = crate::tests::setup();
    let entry = Entry::default();

    let mut vault = crate::tests::get_test_vault();
    assert_eq!(vault.write_entry("foo/bar", &entry).is_ok(), true);

    let vault = Vault::open().expect("could not read vault");
    let retrieved = vault.read_entry("foo/bar");

    assert_eq!(retrieved.is_ok(), true);
    assert_eq!(retrieved.unwrap(), entry);
  }
}
