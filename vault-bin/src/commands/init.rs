use std::error::Error;

use log::*;

use vault::prelude::*;

use crate::util::vault_path;

pub(crate) fn init(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let identity = args.value_of("identity").unwrap();

  VaultHandle::create(vault_path(), identity)?.write()?;

  info!("vault initialized successfully");

  Ok(())
}

#[cfg(test)]
mod tests {
  use std::env;

  use clap::App;

  use vault::prelude::*;

  #[test]
  fn init() {
    let tmp = crate::spec::setup();

    let yml = load_yaml!("../cli.yml");
    let app = App::from_yaml(yml).get_matches_from(vec!["", "init", crate::spec::GPG_IDENTITY]);

    if let ("init", Some(args)) = app.subcommand() {
      assert_eq!(super::init(args).is_ok(), true);
      assert_eq!(super::init(args).is_err(), true);

      let handle = VaultHandle::open(tmp.path()).expect("could not get vault metadata");

      assert_eq!(handle.vault.get_identity(), "vault@apognu.github.com");

      return;
    }

    panic!("command init not triggering");
  }
}
