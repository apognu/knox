use std::error::Error;

use log::*;

use crate::prelude::*;

pub(crate) fn init(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let identity = args.value_of("identity").unwrap();

  Vault::create(identity)?.write()?;

  info!("vault initialized successfully");

  Ok(())
}

#[cfg(test)]
mod tests {
  use clap::App;

  use crate::prelude::*;

  #[test]
  fn init() {
    let _tmp = crate::tests::setup();

    let yml = load_yaml!("../cli.yml");
    let app = App::from_yaml(yml).get_matches_from(vec!["", "init", crate::tests::GPG_IDENTITY]);

    if let ("init", Some(args)) = app.subcommand() {
      assert_eq!(super::init(args).is_ok(), true);
      assert_eq!(super::init(args).is_err(), true);

      let vault = Vault::open().expect("could not get vault metadata");

      assert_eq!(vault.get_identity(), "vault@apognu.github.com");

      return;
    }

    panic!("command init not triggering");
  }
}
