use std::error::Error;

use colored::*;
use log::*;

use libknox::prelude::*;

use crate::util::vault_path;

pub(crate) fn init(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let path = vault_path()?;
  let identities: Vec<String> = args
    .values_of("identity")
    .unwrap()
    .map(|s| s.to_string())
    .collect();

  VaultContext::create(&path, &identities)?.write()?;

  info!("vault initialized successfully at {}", path.bold());

  Ok(())
}

#[cfg(test)]
mod tests {
  use clap::App;

  use libknox::prelude::*;
  use knox_testing::spec;

  #[test]
  fn init() {
    let tmp = spec::setup();

    let yml = load_yaml!("../cli.yml");
    let app = App::from_yaml(yml).get_matches_from(vec!["", "init", spec::GPG_IDENTITY]);

    if let ("init", Some(args)) = app.subcommand() {
      assert_eq!(super::init(args).is_ok(), true);
      assert_eq!(super::init(args).is_err(), true);

      let context = VaultContext::open(tmp.path()).expect("could not get vault metadata");

      assert_eq!(
        context.vault.get_identities(),
        &["vault-test@apognu.github.com".to_string()]
      );

      return;
    }

    panic!("command init not triggering");
  }
}
