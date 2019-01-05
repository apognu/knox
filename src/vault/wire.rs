use std::error::Error;
use std::fs::{create_dir_all, File};
use std::path::Path;

use protobuf::{parse_from_bytes, CodedOutputStream, Message};

use crate::pb;
use crate::util::{normalize_path, GenericError};
use crate::vault::{gpg, pack};
use sha3::{Digest, Sha3_256};
use uuid::Uuid;

pub(crate) const BASE_PATH: &str = "/tmp/vault";
pub(crate) const METADATA_FILE: &str = "_vault.meta";

pub(crate) fn get_vault() -> Result<pb::Vault, Box<dyn Error>> {
  if !Path::new(&normalize_path(&METADATA_FILE)).exists() {
    return Err(GenericError::throw(&format!(
      "vault does not exist at {}, please initialize it",
      normalize_path(&""),
    )));
  }

  let pack = super::gpg::decrypt(&mut File::open(normalize_path(&METADATA_FILE))?)?;
  let vault = parse_from_bytes::<pb::Vault>(&pack)?;

  Ok(vault)
}

pub(crate) fn pack<M>(message: &M) -> Result<Vec<u8>, Box<dyn Error>>
where
  M: Message,
{
  let mut pack = Vec::new();
  let mut cos = CodedOutputStream::new(&mut pack);

  message.write_to(&mut cos)?;
  cos.flush()?;

  Ok(pack)
}

pub(crate) fn create_metadata(identity: &str) -> Result<pb::Vault, Box<dyn Error>> {
  if pack::exists(METADATA_FILE) {
    return Err(GenericError::throw(
      "a vault already exists, refusing to overwrite",
    ));
  }

  gpg::get_keys(&mut gpg::get_context()?, identity)?;

  let vault = pb::Vault {
    identity: identity.to_string(),
    ..pb::Vault::default()
  };

  Ok(vault)
}

pub(crate) fn write_metadata(vault: &pb::Vault) -> Result<(), Box<dyn Error>> {
  create_dir_all(normalize_path(&""))?;
  pack::write(&vault, METADATA_FILE, &pack(vault)?)?;

  Ok(())
}

pub(crate) fn add_index(vault: &mut pb::Vault, path: &str, destination: &str) {
  vault
    .mut_index()
    .insert(path.to_string(), destination.to_string());
}

pub(crate) fn remove_index(vault: &mut pb::Vault, path: &str) {
  vault.mut_index().remove(path);
}

pub(crate) fn create_parents<T>(path: &T) -> Result<(), Box<dyn Error>>
where
  T: AsRef<Path>,
{
  let path = normalize_path(path);
  let mut path = path.split('/').collect::<Vec<&str>>();
  path.pop();

  create_dir_all(path.join("/"))?;

  Ok(())
}

pub(crate) fn hash_path<T>(path: T, salt: Option<&str>) -> (String, String)
where
  T: AsRef<Path>,
{
  let salt = match salt {
    None => Uuid::new_v4().to_hyphenated().to_string(),
    Some(salt) => salt.to_string(),
  };

  let mut hasher = Sha3_256::new();
  hasher.input(&salt);
  hasher.input(format!("{}", path.as_ref().display()));

  let hash = format!("{:x}", hasher.result());

  (salt, format!("{}/{}", &hash[0..2], hash))
}

#[cfg(test)]
mod tests {
  use super::pb;
  use crate::vault::pack;

  #[test]
  fn create_metadata() {
    let _tmp = crate::tests::setup();

    super::write_metadata(
      &super::create_metadata(crate::tests::GPG_IDENTITY).expect("could not create vault"),
    )
    .expect("could not write metadata");

    let vault = super::get_vault().expect("could not retrieve vault");

    assert_eq!(crate::tests::GPG_IDENTITY, vault.get_identity());
  }

  #[test]
  fn read_and_write_metadata() {
    let _tmp = crate::tests::setup();

    let vault = pb::Vault {
      identity: crate::tests::GPG_IDENTITY.to_string(),
      ..pb::Vault::default()
    };

    super::write_metadata(&vault).expect("could not write pack");
    let retrieved: pb::Vault = super::get_vault().expect("could not read pack");

    assert_eq!(vault, retrieved);
  }

  #[test]
  fn add_index() {
    let _tmp = crate::tests::setup();
    let mut vault =
      super::create_metadata(crate::tests::GPG_IDENTITY).expect("could not create vault");

    super::write_metadata(&mut vault).expect("could not write metadata");
    super::add_index(&mut vault, "foo/bar", "lorem/ipsum");
    super::write_metadata(&mut vault).expect("could not write metadata");

    let vault = super::get_vault().expect("coult not get vault");

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
    let mut vault =
      super::create_metadata(crate::tests::GPG_IDENTITY).expect("could not create vault");

    vault
      .mut_index()
      .insert("foo/bar".to_string(), "lorem/ipsum".to_string());

    super::write_metadata(&vault).expect("could not write metadata");
    super::remove_index(&mut vault, "foo/bar");
    super::write_metadata(&vault).expect("could not write metadata");

    let vault = super::get_vault().expect("could not retrieve metadata");

    assert_eq!(None, vault.get_index().get("foo/bar"));
  }

  #[test]
  fn create_parents() {
    let _tmp = crate::tests::setup();

    super::create_parents(&"foo/bar/lorem/ipsum").expect("could not create directories");

    assert_eq!(pack::exists("foo/bar/lorem"), true);
    assert_eq!(pack::exists("hello/world"), false);
  }

  #[test]
  fn hash_path() {
    let result = super::hash_path("foo/bar/lorem/ipsum", Some("test"));

    assert_eq!(
      (
        "test".to_string(),
        "5f/5f04f2a16436fac1c160fa9a618e20aed77591b49fc5464763c27b585bf82a2f".to_string()
      ),
      result
    );

    assert_eq!(
      result,
      super::hash_path("foo/bar/lorem/ipsum", Some("test"))
    );
  }
}
