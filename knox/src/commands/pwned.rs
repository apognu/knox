use std::collections::HashMap;
use std::error::Error;

use colored::*;
use indicatif::ProgressBar;
use libknox::prelude::*;
use log::*;
use sha1::{Digest, Sha1};

use crate::util::vault_path;

#[derive(PartialEq, Debug)]
pub(crate) enum PwnedResult {
  Clear,
  Pwned,
  Error,
}

pub(crate) fn pwned(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  if args.is_present("path") {
    check_entry(args)
  } else {
    check_vault(args)
  }
}

fn check_entry(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let vault = VaultContext::open(vault_path()?)?;
  let path = args.value_of("path").unwrap();
  let entry = vault.read_entry(path)?;
  let pwnage = check_attributes(&entry.attributes);

  info!("Pwnage status for attributes at {}", path.bold());

  for (name, attribute) in pwnage {
    match attribute {
      PwnedResult::Error => println!(
        "  {}  {} -> {} (could not retrieve result)",
        "⋯".magenta(),
        name.dimmed(),
        "ERROR".magenta()
      ),
      PwnedResult::Clear => println!(
        "  {}  {} -> {}",
        "✓".green(),
        name.dimmed(),
        "CLEAR".green()
      ),
      PwnedResult::Pwned => println!("  {}  {} -> {}", "⚠".red(), name.dimmed(), "PWNED".red()),
    }
  }

  Ok(())
}

fn check_vault(_args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let context = VaultContext::open(vault_path()?)?;
  let progress = ProgressBar::new(context.vault.get_index().len() as u64);

  info!("checking for pwned secret across your vault");

  for path in context.vault.get_index().keys() {
    let entry = context.read_entry(&path)?;

    for (name, attribute) in entry.attributes {
      if attribute.confidential && !attribute.file {
        if let AttributeValue::String(value) = attribute.value() {
          if let PwnedResult::Pwned = check(&value) {
            progress.println(format!(
              "  {}  {}/{} -> {}",
              "⚠".red(),
              path.bold(),
              name.dimmed(),
              "PWNED".red()
            ));
          }
        }
      }
    }

    progress.inc(1);
  }

  Ok(())
}

pub(crate) fn check_attributes(
  attributes: &HashMap<String, Attribute>,
) -> Vec<(String, PwnedResult)> {
  attributes
    .iter()
    .filter(|(_, attribute)| attribute.confidential && !attribute.file)
    .flat_map(|(name, attribute)| match attribute.value() {
      AttributeValue::String(value) => Some((name.to_string(), check(&value))),
      _ => None,
    })
    .collect()
}

fn check(value: &str) -> PwnedResult {
  let mut hasher = Sha1::default();
  hasher.input(value);

  let hash = format!("{:x}", hasher.result());
  let prefix = &hash[..5];

  let client = reqwest::Client::new()
    .get(&format!("https://api.pwnedpasswords.com/range/{}", prefix))
    .header("User-Agent", "vault (https://github.com/apognu/vault.rs)")
    .send();

  match client {
    Err(_) => return PwnedResult::Error,
    Ok(mut response) => {
      if response.status() != 200 {
        return PwnedResult::Error;
      }

      match response.text() {
        Err(_) => return PwnedResult::Error,
        Ok(body) => {
          for line in body.lines() {
            let tokens: Vec<&str> = line.split(':').collect();
            if tokens.len() != 2 {
              continue;
            }

            if hash == format!("{}{}", prefix, tokens[0]).to_lowercase() {
              return PwnedResult::Pwned;
            }
          }
        }
      }
    }
  }

  PwnedResult::Clear
}

#[cfg(test)]
mod tests {
  use super::PwnedResult;
  use rand::{distributions::Alphanumeric, Rng};

  #[test]
  fn check() {
    assert_eq!(super::check("azerty"), PwnedResult::Pwned);

    let secure = rand::thread_rng()
      .sample_iter(&Alphanumeric)
      .take(64)
      .collect::<String>();

    // Dangerous, this could fail CI if random string is actually pwned
    assert_eq!(super::check(&secure), PwnedResult::Clear);
  }
}
