use std::error::Error;
use std::fs::{create_dir_all, read_dir, remove_dir, remove_file, File, OpenOptions};
use std::io::Write;
use std::path::Path;

use protobuf::parse_from_bytes;

use super::Packing;
use crate::gpg;
use crate::pb::*;
use crate::util::{self, VaultError};

pub struct VaultContext {
  pub path: String,
  pub vault: Vault,
}

impl VaultContext {
  pub fn create<P>(path: P, identities: &[String]) -> Result<Self, Box<dyn Error>>
  where
    P: AsRef<Path>,
  {
    if path.as_ref().join(util::METADATA_FILE).exists() {
      return Err(VaultError::throw(&format!(
        "a vault already exists at {}, refusing to overwrite",
        path.as_ref().display(),
      )));
    }

    if path.as_ref().exists() {
      match read_dir(&path) {
        Err(err) => return Err(Box::new(err)),
        Ok(directory) => {
          if directory.count() > 0 {
            return Err(VaultError::throw(&format!(
              "a non-empty directory already exists at {}, refusing to overwrite",
              path.as_ref().display()
            )));
          }
        }
      }
    }

    let vault = Vault {
      identities: protobuf::RepeatedField::from(identities),
      ..Vault::default()
    };

    Ok(Self {
      path: format!("{}", path.as_ref().display()),
      vault,
    })
  }

  pub fn open<P>(path: P) -> Result<Self, Box<dyn Error>>
  where
    P: AsRef<Path>,
  {
    let metadata = path.as_ref().join(util::METADATA_FILE);
    if !metadata.exists() {
      return Err(VaultError::throw(&format!(
        "vault does not exist at {}, please initialize it",
        path.as_ref().display(),
      )));
    }

    let pack = gpg::decrypt(&mut File::open(&metadata)?)?;
    let vault = parse_from_bytes::<Vault>(&pack)?;

    Ok(Self {
      path: format!("{}", path.as_ref().display()),
      vault,
    })
  }

  pub fn write(&self) -> Result<(), Box<dyn Error>> {
    create_dir_all(util::normalize_path(self, &""))?;

    let mut file = OpenOptions::new()
      .create(true)
      .write(true)
      .open(util::normalize_path(self, &util::METADATA_FILE))?;

    file.write_all(&gpg::encrypt(&self.vault, &self.vault.pack()?)?)?;

    Ok(())
  }

  pub fn add_index(&mut self, path: &str, destination: &str) {
    self
      .vault
      .mut_index()
      .insert(path.to_string(), destination.to_string());
  }

  pub fn remove_index(&mut self, path: &str) {
    self.vault.mut_index().remove(path);
  }

  pub fn read_entry(&self, path: &str) -> Result<Entry, Box<dyn Error>> {
    if !self.vault.get_index().contains_key(path) {
      return Err(VaultError::throw("no entry was found at this path"));
    }

    let path = util::hash_path(self.vault.get_index().get(path));
    let entry = Entry::read(self, path)?;

    Ok(entry)
  }

  pub fn write_entry(&mut self, path: &str, entry: &Entry) -> Result<(), Box<dyn Error>> {
    let hash = util::hash_path(self.vault.get_index().get(path));

    util::create_parents(self, &hash)?;

    let mut file = OpenOptions::new()
      .create(true)
      .truncate(true)
      .write(true)
      .open(&util::normalize_path(self, &hash))?;

    file.write_all(&gpg::encrypt(&self.vault, &entry.pack()?)?)?;

    self.add_index(path, &hash);
    self.write()?;

    Ok(())
  }

  pub fn delete_entry(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
    if let Some(salt) = self.vault.get_index().get(path) {
      let hash = util::hash_path(Some(salt));

      self.write()?;
      remove_file(util::normalize_path(self, &hash))?;

      for directory in Path::new(&hash).ancestors() {
        let _ = remove_dir(util::normalize_path(
          self,
          &format!("{}", directory.display()),
        ));
      }

      self.remove_index(&path);

      return Ok(());
    }

    Err(VaultError::throw(
      "requested entry does not exist in the vault",
    ))
  }

  pub fn has_pack<P>(&self, path: P) -> bool
  where
    P: AsRef<Path>,
  {
    Path::new(&util::normalize_path(self, &path)).exists()
  }

  pub fn add_identity(&mut self, identity: &str) {
    self.vault.mut_identities().push(identity.to_string());
  }

  pub fn remove_identity(&mut self, identity: &str) {
    let identities = self
      .vault
      .identities
      .iter()
      .filter(|id| id != &identity)
      .map(|id| id.to_owned())
      .collect::<Vec<String>>();

    self
      .vault
      .set_identities(protobuf::RepeatedField::from(identities))
  }
}

#[cfg(test)]
mod tests {
  use knox_testing::spec;

  use crate::prelude::*;
  use crate::util;

  #[test]
  fn create() {
    let tmp = spec::setup();

    VaultContext::create(tmp.path(), &spec::get_test_identities())
      .expect("could not create vault")
      .write()
      .expect("could not write metadata");

    let context = VaultContext::open(&tmp.path()).expect("could not retrieve vault");

    assert_eq!(spec::get_test_identities(), context.vault.get_identities());
  }

  #[test]
  fn read_and_write() {
    let tmp = spec::setup();
    let context = crate::spec::get_test_vault(tmp.path()).expect("could not get vault");

    context.write().expect("could not write pack");
    let retrieved = VaultContext::open(tmp.path()).expect("could not read pack");

    assert_eq!(context.vault, retrieved.vault);
  }

  #[test]
  fn add_index() {
    let tmp = spec::setup();
    let mut context = crate::spec::get_test_vault(tmp.path()).expect("could not get vault");

    context.write().expect("could not write metadata");
    context.add_index("foo/bar", "lorem/ipsum");
    context.write().expect("could not write metadata");

    let retrieved = VaultContext::open(tmp.path()).expect("coult not get vault");

    assert_eq!(
      "lorem/ipsum",
      retrieved
        .vault
        .get_index()
        .get("foo/bar")
        .expect("could not find index")
    );
  }

  #[test]
  fn remove_index() {
    let tmp = spec::setup();
    let mut context = crate::spec::get_test_vault(tmp.path()).expect("could not get vault");

    context
      .vault
      .mut_index()
      .insert("foo/bar".to_string(), "lorem/ipsum".to_string());

    context.write().expect("could not write metadata");
    context.remove_index("foo/bar");
    context.write().expect("could not write metadata");

    let retrieved = VaultContext::open(tmp.path()).expect("could not retrieve metadata");

    assert_eq!(None, retrieved.vault.get_index().get("foo/bar"));
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
    let tmp = spec::setup();
    let context = crate::spec::get_test_vault(tmp.path()).expect("could not get vault");

    std::fs::File::create(util::normalize_path(&context, &"test")).expect("could not create file");

    assert_eq!(true, context.has_pack("test"));
    assert_eq!(false, context.has_pack("foobar"));
  }

  #[test]
  fn read_and_write_entry() {
    let tmp = spec::setup();
    let mut vault = crate::spec::get_test_vault(tmp.path()).expect("could not get vault");

    let entry = Entry::default();

    assert_eq!(vault.write_entry("foo/bar", &entry).is_ok(), true);

    let retrieved = VaultContext::open(tmp.path())
      .expect("could not read vault")
      .read_entry("foo/bar");

    assert_eq!(retrieved.is_ok(), true);
    assert_eq!(retrieved.unwrap(), entry);
  }
}
