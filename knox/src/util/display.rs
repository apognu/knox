use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use libknox::{totp, *};

pub(crate) fn entry(path: &str, entry: &Entry, print: bool) {
  use colored::*;

  let mut components: Vec<&str> = path.split('/').collect();
  let file_name = components.pop().unwrap();

  let mut crumbs = components.iter().map(|component| format!("{}", component.blue())).collect::<Vec<String>>();

  crumbs.insert(0, "🔒 Knox".to_string());

  print!("{}", crumbs.join(&format!("{}", " / ".dimmed())));
  println!(" {} {}", "/".dimmed(), file_name.bold());

  let length = Cell::new(0);
  let mut attributes: Vec<(String, String)> = entry
    .get_attributes()
    .iter()
    .map(|(key, attribute)| {
      if key.len() > length.get() {
        length.set(key.len());
      }

      match (attribute.confidential, attribute.file, print) {
        (true, false, false) => (key.to_string(), format!("{}", "<redacted>".red())),
        (true, false, true) => (key.to_string(), format!("{}", attribute.value.red())),
        (false, true, _) => (key.to_string(), format!("{}", "<file content>".green())),
        _ => (key.to_string(), attribute.value.clone()),
      }
    })
    .collect();

  match (entry.has_totp(), print) {
    (true, false) => attributes.push((String::from("@totp"), format!("{}", "<redacted>".red()))),
    (true, true) => {
      if let Ok((totp, expiration)) = totp::get_totp(&entry, None) {
        let expires_in = expiration - SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        attributes.push((String::from("@totp"), format!("{} {}", totp.blue(), format!("(expires in {}s)", expires_in).dimmed())));
      }
    }
    _ => (),
  }

  for (key, value) in attributes {
    println!("{: >width$} = {}", key.bold(), value, width = length.get() + 2);
  }
}

pub(crate) fn write_files<T>(path: T, entry: &Entry, filter: &Option<Vec<&str>>) -> Result<(), Box<dyn Error>>
where
  T: AsRef<Path> + fmt::Display,
{
  let dir = path.as_ref().to_str().unwrap().replace("/", "-");
  let dir = format!("vault-{}", dir);
  let path = Path::new(&dir);

  if path.exists() {
    return Err(VaultError::throw(&format!("'{}' already exists in the current directory", dir)));
  }

  fs::create_dir(&dir)?;

  let attributes = entry.get_attributes().iter().filter(|(key, _)| match &filter {
    None => true,
    Some(filter) => filter.contains(&key.as_ref()),
  });

  for (key, attribute) in attributes {
    let mut file = OpenOptions::new().create(true).truncate(true).write(true).open(format!("{}/{}", &dir, key))?;

    match attribute.value() {
      AttributeValue::String(string) => file.write_all(string.as_bytes())?,
      AttributeValue::Binary(bytes) => file.write_all(&bytes)?,
    }
  }

  Ok(())
}
