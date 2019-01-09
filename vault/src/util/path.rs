use std::error::Error;
use std::fs::create_dir_all;
use std::path::Path;

use uuid::Uuid;

use crate::prelude::*;

pub(crate) const METADATA_FILE: &str = "_vault.meta";

pub(crate) fn create_parents<T>(handle: &VaultHandle, path: &T) -> Result<(), Box<dyn Error>>
where
  T: AsRef<Path>,
{
  let path = normalize_path(handle, path);
  let mut path = path.split('/').collect::<Vec<&str>>();
  path.pop();

  create_dir_all(path.join("/"))?;

  Ok(())
}

pub(crate) fn normalize_path<T>(handle: &VaultHandle, path: &T) -> String
where
  T: AsRef<Path>,
{
  format!("{}/{}", handle.path, path.as_ref().display())
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
  #[test]
  fn create_parents() {
    let tmp = crate::spec::setup();
    let handle = crate::spec::get_test_vault(tmp.path()).expect("could not get vault");

    super::create_parents(&handle, &"foo/bar/lorem/ipsum").expect("could not create directories");

    assert_eq!(handle.has_pack("foo/bar/lorem"), true);
    assert_eq!(handle.has_pack("hello/world"), false);
  }

  #[test]
  fn normalize_path() {
    let tmp = crate::spec::setup();
    let handle = crate::spec::get_test_vault(tmp.path()).expect("could not get vault");

    assert_eq!(
      super::normalize_path(&handle, &"lorem/ipsum"),
      format!("{}/{}", tmp.path().display(), "lorem/ipsum"),
    );
  }

  #[test]
  fn hash_path() {
    assert_eq!(super::hash_path(Some(&"test".to_string())), "test");
    assert_eq!(super::hash_path(None).len(), 39);
    assert_ne!(super::hash_path(None), super::hash_path(None));
  }
}
