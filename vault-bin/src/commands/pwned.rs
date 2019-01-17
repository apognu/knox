use std::error::Error;

use colored::*;
use log::*;
use sha1::{Digest, Sha1};
use vault::prelude::*;

use crate::util::vault_path;

#[derive(PartialEq, Debug)]
enum PwnedResult {
  Clear,
  Pwned,
  Error,
}

pub(crate) fn pwned(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let vault = VaultHandle::open(vault_path()?)?;
  let path = args.value_of("path").unwrap();
  let entry = vault.read_entry(path)?;

  let result = entry
    .attributes
    .iter()
    .filter(|(_, attribute)| attribute.confidential && !attribute.file)
    .flat_map(|(name, attribute)| match attribute.value() {
      AttributeValue::String(value) => Some((name, is_pwned(&value))),
      _ => None,
    });

  info!("Pwnage status for attributes at {}", path.bold());

  for (name, attribute) in result {
    match attribute {
      PwnedResult::Error => println!(
        "  {} {} -> {} (could not retrieve result)",
        "⋯".magenta(),
        name.dimmed(),
        "ERROR".magenta()
      ),
      PwnedResult::Clear => println!(
        "  {} {} -> {}",
        "✓".green(),
        name.dimmed(),
        "CLEAR".green()
      ),
      PwnedResult::Pwned => println!("  {} {} -> {}", "⚠".red(), name.dimmed(), "PWNED".red()),
    }
  }

  Ok(())
}

fn is_pwned(value: &str) -> PwnedResult {
  let mut hasher = Sha1::default();
  hasher.input(value);

  let hash = format!("{:x}", hasher.result());
  let prefix = &hash[..5];
  let url = &format!("https://api.pwnedpasswords.com/range/{}", prefix);

  match reqwest::get(url) {
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
  fn is_pwned() {
    assert_eq!(super::is_pwned("azerty"), PwnedResult::Pwned);

    let secure = rand::thread_rng()
      .sample_iter(&Alphanumeric)
      .take(64)
      .collect::<String>();

    // Dangerous, this could fail CI if random string is actually pwned
    assert_eq!(super::is_pwned(&secure), PwnedResult::Clear);
  }
}
