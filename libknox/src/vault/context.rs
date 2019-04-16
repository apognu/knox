//! Handle around a [Vault](struct.Vault.html) instance.

use std::error::Error;
use std::fs::{create_dir_all, read_dir, remove_dir, remove_file, File, OpenOptions};
use std::io::Write;
use std::path::Path;

use protobuf::parse_from_bytes;

use super::pack::Packing;
use crate::gpg;
use crate::pb::*;
use crate::util::{self, git, VaultError};

/// Handle around a [Vault](struct.Vault.html) instance.
pub struct VaultContext {
  pub path: String,
  pub vault: Vault,
}

impl VaultContext {
  /// Create a new vault.
  ///
  /// Initializes a new empty vault, encrypted with the provided GPG
  /// identities. This function will fail if the given `path` is not empty or
  /// if a public key matching the `identity` cannot be found.
  ///
  /// # Arguments
  ///
  ///  * `path`       - the filesystem path at which to create the vault.
  ///  * `identities` - a slice if GPG identities, represented by their owner's
  ///                   email address.
  pub fn create<P>(path: P, identities: &[String]) -> Result<Self, Box<dyn Error>>
  where
    P: AsRef<Path>,
  {
    let path = path.as_ref();

    if path.join(util::METADATA_FILE).exists() {
      return Err(VaultError::throw(&format!(
        "a vault already exists at {}, refusing to overwrite",
        path.display(),
      )));
    }

    if path.exists() {
      match read_dir(path) {
        Err(err) => return Err(Box::new(err)),
        Ok(directory) => {
          if directory.count() > 0 {
            return Err(VaultError::throw(&format!(
              "a non-empty directory already exists at {}, refusing to overwrite",
              path.display()
            )));
          }
        }
      }
    }

    let vault = Self {
      path: format!("{}", path.display()),
      vault: Vault {
        identities: protobuf::RepeatedField::from(identities),
        ..Vault::default()
      },
    };

    Ok(vault)
  }

  /// Return a handle to a [Vault](struct.Vault.html) from the filesystem.
  ///
  /// Opens, decrypt the metadata of, and returns a handle that allows you to
  /// manipulate a [Vault](struct.Vault.html). This function will fail of the
  /// given `path` is not a vault instance or if it cannot be decrypted with
  /// an available GPG private key.
  ///
  /// # Arguments
  ///
  ///  * `path` - filesystem path where the vault is located.
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

  /// Write the vault metadata.
  ///
  /// Persists all changes to the vault's metadata into the `_knox.meta` file
  /// containing the encrypted mapping between virtual (user) secret paths and
  /// filesystem paths.
  ///
  /// This requires the GPG public keys of all identities used in the vault.
  pub fn write(&self) -> Result<(), Box<dyn Error>> {
    create_dir_all(util::normalize_path(self, &""))?;

    let mut file = OpenOptions::new()
      .create(true)
      .write(true)
      .open(util::normalize_path(self, &util::METADATA_FILE))?;

    file.write_all(&gpg::encrypt(&self.vault, &self.vault.pack()?)?)?;

    Ok(())
  }

  /// Add an [Entry](struct.Entry.html) to the index.
  ///
  /// Adds an [Entry](struct.Entry.html) to the index of a vault, allowing to
  /// retrieve a filesystem path from a virtual path. This does not manage the
  /// secret itself.
  ///
  /// To persist the change, refer to
  /// [VaultContext::write](struct.VaultContext.html#method.write).
  ///
  /// # Arguments
  ///
  ///  * `path`        - virtual path to the entry.
  ///  * `destination` - physical filesystem path to the entry.
  pub fn add_index(&mut self, path: &str, destination: &str) {
    self
      .vault
      .mut_index()
      .insert(path.to_string(), destination.to_string());
  }

  /// Remove an [Entry](struct.Entry.html) from the index.
  ///
  /// Removes an [Entry](struct.Entry.html) to the index of a vault, allowing
  /// to retrieve a filesystem path from a virtual path. This does not manage
  /// the secret itself.
  ///
  /// To persist the change, refer to
  /// [VaultContext::write](struct.VaultContext.html#method.write).
  ///
  /// # Arguments
  ///
  ///  * `path` - virtual path to the entry
  pub fn remove_index(&mut self, path: &str) {
    self.vault.mut_index().remove(path);
  }

  /// Read an [Entry](struct.Entry.html).
  ///
  /// Takes a virtual path and returns the decrypted [Entry](struct.Entry.html)
  /// from the vault, if it exists.
  ///
  /// # Arguments
  ///
  ///  * `path` - the virtual path to the entry.
  pub fn read_entry(&self, path: &str) -> Result<Entry, Box<dyn Error>> {
    if !self.vault.get_index().contains_key(path) {
      return Err(VaultError::throw("no entry was found at this path"));
    }

    let path = util::hash_path(self.vault.get_index().get(path));
    let entry = Entry::read(self, path)?;

    Ok(entry)
  }

  /// Persist an [Entry](struct.Entry.html).
  ///
  /// Encrypts and writes an [Entry](struct.Entry.html) to its physical
  /// location as described in the vault's index. The entry must exist in the
  /// index beforehand.
  ///
  /// This requires the GPG public keys of all identities used in the vault.
  ///
  /// # Arguments
  ///
  ///  * `path`  - the virtual path to the entry.
  ///  * `entry` - the `Entry` to be written.
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

  /// Delete an [Entry](struct.Entry.html).
  ///
  /// Deletes an [Entry](struct.Entry.html) both from its backing filesystem
  /// location and from the index.
  ///
  /// This requires the GPG public keys of all identities used in the vault
  /// because the index needs to be updated.
  ///
  /// # Arguments
  ///
  ///  * `path` - the virtual path to the entry.
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
      self.write()?;

      return Ok(());
    }

    Err(VaultError::throw(
      "requested entry does not exist in the vault",
    ))
  }

  /// Check if a file exists under the vault's directory.
  ///
  /// # Arguments
  ///
  ///  * - `path` - a path relative to the vault's root directory.
  pub fn has_pack<P>(&self, path: P) -> bool
  where
    P: AsRef<Path>,
  {
    Path::new(&util::normalize_path(self, &path)).exists()
  }

  /// Add an identity to the vault
  ///
  /// Saves a new identity to the vault's metadata. This does not touch any
  /// existing secrets, which should be re-encrypted for the new identity to
  /// be used.
  ///
  /// # Arguments
  ///
  ///  * `identity` - the GPG identity
  pub fn add_identity(&mut self, identity: &str) {
    self.vault.mut_identities().push(identity.to_string());
  }

  /// Remove an identity from the vault.
  ///
  /// Removes an existing identity from the vault's metadata. This does not
  /// touch any existing secrets, which should be re-encrypted for the
  /// removed identity to be unable de decrypt them.
  ///
  /// # Arguments
  ///
  ///  * `identity` - the GPG identity
  pub fn remove_identity(&mut self, identity: &str) {
    let identities = self
      .vault
      .identities
      .iter()
      .filter(|id| id != &identity)
      .map(std::borrow::ToOwned::to_owned)
      .collect::<Vec<String>>();

    self
      .vault
      .set_identities(protobuf::RepeatedField::from(identities))
  }

  /// Initialize a local git repository
  pub fn git_init(&self) -> Result<(), Box<dyn Error>> {
    git::init(&self)
  }

  /// Commit all unstaged files to git repository
  ///
  /// Use this function to add all uncommitted modifications to the git index
  /// and commit them into the local git repository.
  ///
  /// # Arguments
  ///
  ///  * `message` - the commit message to be used for the commit
  pub fn commit(&self, message: &str) -> Result<(), Box<dyn Error>> {
    git::commit(&self, message)
  }

  /// Set the URL of the remote git repository
  ///
  /// No particular check is performed on the validity of the provided URL.
  ///
  /// # Arguments
  ///
  ///  * `origin` - the URL for the remote git repository
  pub fn set_git_origin(&self, origin: &str) -> Result<(), Box<dyn Error>> {
    git::set_origin(&self, origin)
  }

  /// Push all commited data to the remote git repository
  pub fn push(&self) -> Result<(), Box<dyn Error>> {
    git::push(&self)
  }
}

#[cfg(test)]
mod tests {
  use knox_testing::spec;

  use crate::*;

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
