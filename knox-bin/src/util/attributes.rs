use std::collections::HashMap;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Read;

use knox::prelude::*;
use rand::{distributions::Alphanumeric, seq::SliceRandom, Rng};

pub(crate) fn build(args: &clap::ArgMatches) -> Result<HashMap<String, Attribute>, Box<dyn Error>> {
  let mut attributes: HashMap<String, Attribute> = HashMap::new();

  for attribute in args.values_of("attributes").unwrap_or_default() {
    let attribute: Vec<&str> = attribute.splitn(2, '=').collect();
    if attribute.len() < 2 {
      return Err(VaultError::throw("could not parse attributes"));
    }

    let (key, value) = (&attribute[0], &attribute[1]);
    let mut attribute = Attribute {
      value: value.to_string(),
      ..Attribute::default()
    };

    if value == &"-" {
      let (length, symbols) = (
        args.value_of("random_length").unwrap_or("16"),
        args.is_present("random_symbols"),
      );

      attribute.value = random_secret(length, symbols)?;
      attribute.confidential = true;
    }
    if value == &"" {
      attribute.value = prompt_for_secret(key)?;
      attribute.confidential = true;
    }
    if value.starts_with('@') {
      let mut file_chars = value.chars();
      file_chars.next();

      let mut file = OpenOptions::new().read(true).open(file_chars.as_str())?;
      let mut buffer: Vec<u8> = Vec::new();
      file.read_to_end(&mut buffer)?;

      attribute.value = String::new();
      attribute.bytes_value = buffer;
      attribute.file = true;
    }

    attributes.insert(key.to_string(), attribute);
  }

  Ok(attributes)
}

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789)(*&^%$#@!~";

fn random_secret(length: &str, symbols: bool) -> Result<String, Box<dyn Error>> {
  let length = length.parse::<usize>().unwrap_or(16);

  if symbols {
    let mut rng = rand::thread_rng();

    Ok(
      (0..length)
        .map(|_| *CHARSET.choose(&mut rng).unwrap() as char)
        .collect::<String>(),
    )
  } else {
    Ok(
      rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .collect::<String>(),
    )
  }
}

fn prompt_for_secret(key: &str) -> Result<String, Box<dyn Error>> {
  use colored::*;

  let secret = rpassword::prompt_password_stdout(&format!("Enter value for '{}': ", key.bold()))?;

  Ok(secret)
}
