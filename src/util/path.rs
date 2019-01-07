use std::env;
use std::error::Error;
use std::fs::create_dir_all;
use std::path::Path;

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

pub(crate) fn hash_path(salt: Option<&String>) -> String {
  match salt {
    None => {
      let uuid = Uuid::new_v4().to_hyphenated().to_string();

      format!("{}/{}", &uuid[..2], &uuid)
    }
    Some(salt) => salt.clone(),
  }
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
    let result = super::hash_path(Some(&"test".to_string()));

    assert_eq!(result, "test");
  }
}
