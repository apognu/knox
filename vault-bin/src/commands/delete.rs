use std::error::Error;

use colored::*;
use log::*;

use vault::prelude::*;

use crate::util::vault_path;

pub(crate) fn delete(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let mut vault = VaultContext::open(vault_path()?)?;
  let path = args.value_of("path").unwrap();

  vault.delete_entry(path)?;
  vault.write()?;

  info!(
    "entry {} was successfully deleted from the vault",
    path.bold()
  );

  Ok(())
}

#[cfg(test)]
mod tests {
  use clap::App;

  use vault::prelude::*;
  use vault_testing::spec;

  #[test]
  fn delete() {
    let tmp = spec::setup();
    let mut context = crate::spec::get_test_vault(tmp.path()).expect("could not get vault");

    let mut entry = Entry::default();
    entry.add_attribute("apikey", "abcdef");

    context
      .write_entry("foo/bar", &entry)
      .expect("could not write entry");

    let yml = load_yaml!("../cli.yml");
    let app = App::from_yaml(yml).get_matches_from(vec!["", "delete", "foo/bar"]);

    if let ("delete", Some(args)) = app.subcommand() {
      assert_eq!(super::delete(args).is_ok(), true);

      let vault = VaultContext::open(tmp.path()).expect("could not open vault");

      assert_eq!(vault.read_entry("foo/bar").is_err(), true);

      return;
    }

    panic!("command delete not triggering");
  }
}
