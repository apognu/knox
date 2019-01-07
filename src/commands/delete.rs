use std::error::Error;

use colored::*;
use log::*;

use crate::prelude::*;

pub(crate) fn delete(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let mut vault = Vault::open()?;
  let path = args.value_of("path").unwrap();

  vault.delete_entry(path)?;
  vault.write()?;

  info!("entry {} was successfully deleted from the vault", path.bold());

  Ok(())
}

#[cfg(test)]
mod tests {
  use clap::App;
  use std::collections::HashMap;

  use crate::prelude::*;

  #[test]
  fn delete() {
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

            map
          },
          ..Entry::default()
        },
      )
      .expect("could not write entry");

    let yml = load_yaml!("../cli.yml");
    let app = App::from_yaml(yml).get_matches_from(vec!["", "delete", "foo/bar"]);

    if let ("delete", Some(args)) = app.subcommand() {
      assert_eq!(super::delete(args).is_ok(), true);

      let vault = Vault::open().expect("could not open vault");

      assert_eq!(vault.read_entry("foo/bar").is_err(), true);

      return;
    }

    panic!("command delete not triggering");
  }
}
