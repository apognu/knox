pub(crate) mod attributes;
pub(crate) mod display;
pub(crate) mod hierarchy;

use std::env;

pub(crate) fn vault_path() -> String {
  env::var("VAULT_PATH").unwrap_or_else(|_| "/tmp/vault".to_string())
}
