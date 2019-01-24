#[cfg(test)]
use std::error::Error;
use std::path::Path;

use vault_testing::spec;

use crate::prelude::*;

pub(crate) fn get_test_vault<P>(path: P) -> Result<VaultContext, Box<dyn Error>>
where
  P: AsRef<Path>,
{
  let context = VaultContext::create(&path, &spec::get_test_identities())?;
  context.write()?;

  Ok(context)
}
