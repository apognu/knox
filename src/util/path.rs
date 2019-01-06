use std::env;
use std::error::Error;
use std::fs::create_dir_all;
use std::path::Path;

use sha3::{Digest, Sha3_256};
use uuid::Uuid;

pub(crate) const BASE_PATH: &str = "/tmp/vault";
pub(crate) const METADATA_FILE: &str = "_vault.meta";

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

pub(crate) fn normalize_path<T>(path: &T) -> String
where
  T: AsRef<Path>,
{
  let base = env::var("VAULT_PATH").unwrap_or_else(|_| BASE_PATH.to_string());

  format!("{}/{}", base, path.as_ref().display())
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
  use crate::prelude::*;

  #[test]
  fn create_parents() {
    let _tmp = crate::tests::setup();

    super::create_parents(&"foo/bar/lorem/ipsum").expect("could not create directories");

    assert_eq!(Vault::has_pack("foo/bar/lorem"), true);
    assert_eq!(Vault::has_pack("hello/world"), false);
  }

  #[test]
  fn normalize_path() {
    std::env::set_var("VAULT_PATH", "/tmp/foo/bar");

    assert_eq!(
      super::normalize_path(&"lorem/ipsum"),
      "/tmp/foo/bar/lorem/ipsum"
    );
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
