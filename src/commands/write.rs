use std::error::Error;

use colored::*;
use log::*;

use crate::prelude::*;
use crate::util::{self, VaultError};

pub(crate) fn add(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let path = args.value_of("path").unwrap();
  let mut vault = Vault::open()?;

  if vault.get_index().contains_key(path) {
    return Err(VaultError::throw("an entry already exists at this path"));
  }

  let attributes = util::build_attributes(args)?;

  let entry = Entry {
    attributes,
    ..Entry::default()
  };

  vault.write_entry(&path, &entry)?;

  info!("entry {} was successfully added to the vault", path.bold());

  Ok(())
}

pub(crate) fn edit(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let path = args.value_of("path").unwrap();
  let attributes = util::build_attributes(args)?;
  let delete_attributes = args.values_of("delete");

  let mut vault = Vault::open()?;

  if !vault.get_index().contains_key(path) {
    return Err(VaultError::throw("no entry was found at this path"));
  }

  let mut entry = vault.read_entry(&path)?;

  entry.mut_attributes().extend(attributes);
  if let Some(delete_attributes) = delete_attributes {
    for delete_attribute in delete_attributes {
      entry.mut_attributes().remove(delete_attribute);
    }
  }
  vault.write_entry(&path, &entry)?;

  info!("entry {} was successfully edited", path.bold());

  Ok(())
}

pub(crate) fn rename(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let source = args.value_of("source").unwrap();
  let destination = args.value_of("destination").unwrap();
  let mut vault = Vault::open()?;

  if !vault.get_index().contains_key(source) {
    return Err(VaultError::throw("no entry was found at this path"));
  }
  if vault.get_index().contains_key(destination) {
    return Err(VaultError::throw(
      "an entry already exists at this destination",
    ));
  }

  let salt = vault.get_index().get(source).unwrap().clone();

  vault.add_index(destination, &salt);
  vault.remove_index(source);
  vault.write()?;

  info!(
    "entry {} was successfully renamed to {}",
    source.bold(),
    destination.bold()
  );

  Ok(())
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use clap::App;

  use crate::prelude::*;

  #[test]
  fn add() {
    let _tmp = crate::tests::setup();

    crate::tests::get_test_vault()
      .write()
      .expect("could not write tests vault");

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

      let vault = Vault::open().expect("could not open vault");
      let entry = vault
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
    let _tmp = crate::tests::setup();

    let mut vault = crate::tests::get_test_vault();
    vault.write().expect("could not write tests vault");

    vault
      .write_entry(
        "foo/bar",
        &Entry {
          attributes: {
            let mut map = HashMap::new();
            map.insert(
              "apikey".to_string(),
              Attribute {
                value: "abcdef".to_string(),
                ..Attribute::default()
              },
            );
            map.insert(
              "to_be_deleted".to_string(),
              Attribute {
                value: "abcdef".to_string(),
                ..Attribute::default()
              },
            );

            map
          },
          ..Entry::default()
        },
      )
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

      let vault = Vault::open().expect("could not open vault");
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
    let _tmp = crate::tests::setup();

    let mut vault = crate::tests::get_test_vault();
    vault.write().expect("could not write tests vault");

    let entry = &Entry {
      attributes: {
        let mut map = HashMap::new();
        map.insert(
          "apikey".to_string(),
          Attribute {
            value: "abcdef".to_string(),
            ..Attribute::default()
          },
        );
        map.insert(
          "to_be_deleted".to_string(),
          Attribute {
            value: "abcdef".to_string(),
            ..Attribute::default()
          },
        );

        map
      },
      ..Entry::default()
    };

    vault
      .write_entry("foo/bar", &entry)
      .expect("could not write entry");

    let yml = load_yaml!("../cli.yml");

    let app = App::from_yaml(yml).get_matches_from(vec!["", "rename", "foo/bar", "foo/bar"]);

    if let ("rename", Some(args)) = app.subcommand() {
      assert_eq!(super::rename(args).is_err(), true);

      let retrieved = Vault::open()
        .expect("could not open vault")
        .read_entry("foo/bar")
        .expect("could not read entry");

      assert_eq!(entry, &retrieved);

      return;
    }

    let app = App::from_yaml(yml).get_matches_from(vec!["", "rename", "foo/bar", "lorem/ipsum"]);

    if let ("rename", Some(args)) = app.subcommand() {
      assert_eq!(super::rename(args).is_ok(), true);

      let retrieved = Vault::open()
        .expect("could not open vault")
        .read_entry("lorem/ipsum")
        .expect("could not read entry");

      assert_eq!(entry, &retrieved);

      return;
    }

    panic!("command rename not triggering");
  }
}
