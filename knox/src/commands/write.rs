use std::error::Error;

use colored::*;
use libknox::*;
use log::*;

use crate::commands::pwned::{self, PwnedResult};
use crate::util::{self, vault_path};

pub(crate) fn add(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let path = args.value_of("path").unwrap();
  let mut context = VaultContext::open(vault_path()?)?;

  if context.vault.get_index().contains_key(path) {
    return Err(VaultError::throw("an entry already exists at this path"));
  }

  let attributes = util::attributes::build(args)?;
  let pwnage = pwned::check_attributes(&attributes);
  let mut abort = false;

  for pwn in pwnage.iter() {
    if let (name, PwnedResult::Pwned) = pwn {
      warn!(
        "the value for {} has been found in HIBP's data breaches",
        name.bold()
      );
      abort = true
    }
  }

  if abort && !args.is_present("force") {
    return Err(VaultError::throw("aborting because some confidential attributes were breached in Have I Been Pwned, use --force to override"));
  }

  let entry = Entry {
    attributes,
    ..Entry::default()
  };

  context.write_entry(&path, &entry)?;

  info!("entry {} was successfully added to the vault", path.bold());

  context.commit("Added entry.")?;

  Ok(())
}

pub(crate) fn edit(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let path = args.value_of("path").unwrap();
  let attributes = util::attributes::build(args)?;
  let delete_attributes = args.values_of("delete");

  let mut context = VaultContext::open(vault_path()?)?;

  if !context.vault.get_index().contains_key(path) {
    return Err(VaultError::throw("no entry was found at this path"));
  }

  let pwnage = pwned::check_attributes(&attributes);
  let mut abort = false;

  for pwn in pwnage.iter() {
    if let (name, PwnedResult::Pwned) = pwn {
      warn!(
        "the value for {} has been found in HIBP's data breaches",
        name.bold()
      );
      abort = true
    }
  }

  if abort && !args.is_present("force") {
    return Err(VaultError::throw("aborting because some confidential attributes were breached in Have I Been Pwned, use --force to override"));
  }

  let mut entry = context.read_entry(&path)?;

  entry.mut_attributes().extend(attributes);
  if let Some(delete_attributes) = delete_attributes {
    for delete_attribute in delete_attributes {
      entry.mut_attributes().remove(delete_attribute);
    }
  }

  context.write_entry(&path, &entry)?;

  info!("entry {} was successfully edited", path.bold());

  context.commit("Edited entry.")?;

  Ok(())
}

pub(crate) fn rename(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let source = args.value_of("source").unwrap();
  let destination = args.value_of("destination").unwrap();
  let mut context = VaultContext::open(vault_path()?)?;

  if !context.vault.get_index().contains_key(source) {
    return Err(VaultError::throw("no entry was found at this path"));
  }
  if context.vault.get_index().contains_key(destination) {
    return Err(VaultError::throw(
      "an entry already exists at this destination",
    ));
  }

  let salt = context.vault.get_index().get(source).unwrap().clone();

  context.add_index(destination, &salt);
  context.remove_index(source);
  context.write()?;

  info!(
    "entry {} was successfully renamed to {}",
    source.bold(),
    destination.bold()
  );

  context.commit("Renamed entry.")?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use clap::App;

  use knox_testing::spec;
  use libknox::*;

  #[test]
  fn add() {
    let tmp = spec::setup();
    let context = crate::spec::get_test_vault(tmp.path()).expect("could not get vault");

    context.write().expect("could not write tests vault");

    let yml = load_yaml!("../cli.yml");
    let app = App::from_yaml(yml).get_matches_from(vec![
      "",
      "add",
      "foo/bar",
      "username=mitch",
      "password=supersecret",
    ]);

    if let ("add", Some(args)) = app.subcommand() {
      assert_eq!(super::add(args).is_ok(), true);
      assert_eq!(super::add(args).is_ok(), false);

      let context = VaultContext::open(tmp.path()).expect("could not get vault");
      let entry = context
        .read_entry("foo/bar")
        .expect("could not read added entry");

      assert_eq!(
        entry.get_attributes().get("username"),
        Some(&Attribute {
          value: "mitch".to_string(),
          ..Attribute::default()
        })
      );

      assert_eq!(
        entry.get_attributes().get("password"),
        Some(&Attribute {
          value: "supersecret".to_string(),
          ..Attribute::default()
        })
      );

      assert_eq!(entry.get_attributes().get("unknown"), None);

      return;
    }

    panic!("command add not triggering");
  }

  #[test]
  fn edit() {
    let tmp = spec::setup();
    let mut vault = crate::spec::get_test_vault(tmp.path()).expect("could not write tests vault");

    let mut entry = Entry::default();
    entry.add_attribute("apikey", "abcdef");
    entry.add_attribute("to_be_deleted", "abcdef");

    vault
      .write_entry("foo/bar", &entry)
      .expect("could not write entry");

    let yml = load_yaml!("../cli.yml");
    let app = App::from_yaml(yml).get_matches_from(vec![
      "",
      "edit",
      "foo/bar",
      "username=mitch",
      "-d",
      "to_be_deleted",
    ]);

    if let ("edit", Some(args)) = app.subcommand() {
      assert_eq!(super::edit(args).is_ok(), true);

      let vault = VaultContext::open(tmp.path()).expect("could not open vault");
      let entry = vault.read_entry("foo/bar").expect("could not edited entry");

      assert_eq!(
        entry.get_attributes().get("apikey"),
        Some(&Attribute {
          value: "abcdef".to_string(),
          ..Attribute::default()
        })
      );

      assert_eq!(
        entry.get_attributes().get("username"),
        Some(&Attribute {
          value: "mitch".to_string(),
          ..Attribute::default()
        })
      );

      assert_eq!(entry.get_attributes().get("to_be_deleted"), None);

      return;
    }

    panic!("command edit not triggering");
  }

  #[test]
  fn rename() {
    let tmp = spec::setup();
    let mut vault = crate::spec::get_test_vault(tmp.path()).expect("could not write tests vault");

    let mut entry = Entry::default();
    entry.add_attribute("apikey", "abcdef");
    entry.add_attribute("to_be_deleted", "abcdef");

    vault
      .write_entry("foo/bar", &entry)
      .expect("could not write entry");

    let yml = load_yaml!("../cli.yml");

    let app = App::from_yaml(yml).get_matches_from(vec!["", "rename", "foo/bar", "foo/bar"]);

    if let ("rename", Some(args)) = app.subcommand() {
      assert_eq!(super::rename(args).is_err(), true);

      let retrieved = VaultContext::open(tmp.path())
        .expect("could not open vault")
        .read_entry("foo/bar")
        .expect("could not read entry");

      assert_eq!(&entry, &retrieved);

      return;
    }

    let app = App::from_yaml(yml).get_matches_from(vec!["", "rename", "foo/bar", "lorem/ipsum"]);

    if let ("rename", Some(args)) = app.subcommand() {
      assert_eq!(super::rename(args).is_ok(), true);

      let retrieved = VaultContext::open(tmp.path())
        .expect("could not open vault")
        .read_entry("lorem/ipsum")
        .expect("could not read entry");

      assert_eq!(&entry, &retrieved);

      return;
    }

    panic!("command rename not triggering");
  }
}
