use std::cell::Cell;

use crate::pb;

pub(crate) fn entry(path: &str, entry: &pb::Entry) {
  use colored::*;

  let mut components: Vec<&str> = path.split('/').collect();
  let file_name = components.pop().unwrap();

  let mut crumbs = components
    .iter()
    .map(|component| format!("{}", component.blue()))
    .collect::<Vec<String>>();

  crumbs.insert(0, "🔒 Vault store:".to_string());

  print!("{}", crumbs.join(&format!("{}", " / ".dimmed())));
  println!(" / {}", file_name.bold());

  let length = Cell::new(0);
  let attributes = entry.get_attributes().iter().map(|(key, attribute)| {
    if key.len() > length.get() {
      length.set(key.len());
    }

    match (attribute.confidential, attribute.file) {
      (true, false) => (key.to_string(), format!("{}", "<redacted>".red())),
      (false, true) => (key.to_string(), format!("{}", "<file content>".green())),
      _ => (key.to_string(), attribute.value.clone()),
    }
  });

  for (key, value) in attributes {
    for _ in 0..(length.get() - key.len() + 3) {
      print!(" ");
    }

    println!("{} = {}", key.bold(), value);
  }
}
