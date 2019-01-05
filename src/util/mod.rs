pub(crate) mod display;
mod error;
pub(crate) mod hierarchy;

use std::env;
use std::path::Path;

pub(crate) use self::error::*;
use crate::vault::wire::BASE_PATH;

pub(crate) fn normalize_path<T>(path: &T) -> String
where
  T: AsRef<Path>,
{
  let base = env::var("VAULT_PATH").unwrap_or_else(|_| BASE_PATH.to_string());

  format!("{}/{}", base, path.as_ref().display())
}

#[cfg(test)]
mod tests {
  #[test]
  fn normalize_path() {
    std::env::set_var("VAULT_PATH", "/tmp/foo/bar");

    assert_eq!(
      super::normalize_path(&"lorem/ipsum"),
      "/tmp/foo/bar/lorem/ipsum"
    );
  }
}
