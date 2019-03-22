use std::error::Error;

use colored::*;
use knox::prelude::*;

use crate::util::vault_path;

pub(crate) fn info(_args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let path = vault_path()?;
  let context = VaultContext::open(&path)?;

  println!("Vault path: {}", path.bold());

  println!("\nIdentities:");
  for id in context.vault.get_identities() {
    println!("  - {}", id);
  }

  println!(
    "\nNumber of entries: {}",
    context.vault.get_index().len().to_string().bold()
  );

  Ok(())
}
