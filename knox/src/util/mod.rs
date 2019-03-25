pub(crate) mod attributes;
pub(crate) mod display;
pub(crate) mod hierarchy;

use std::env;
use std::error::Error;

use libknox::*;

pub(crate) fn vault_path() -> Result<String, Box<dyn Error>> {
  let path = env::var("KNOX_PATH");

  match path {
    Ok(path) => Ok(path),
    Err(_) => match dirs::home_dir() {
      Some(home) => Ok(format!("{}/.knox", home.display())),
      None => Err(VaultError::throw(
        "could not get your home directory, please set KNOX_PATH",
      )),
    },
  }
}
