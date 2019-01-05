use log::*;
use std::error::Error;

use crate::vault::wire;

pub(crate) fn init(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let identity = args.value_of("identity").unwrap();

  wire::write_metadata(&wire::create_metadata(identity)?)?;

  info!("vault initialized successfully");

  Ok(())
}

#[cfg(test)]
mod tests {
  use clap::App;

  use crate::vault::wire;

  #[test]
  fn init() {
    let _tmp = crate::tests::setup();

    let yml = load_yaml!("../cli.yml");
    let app = App::from_yaml(yml).get_matches_from(vec!["", "init", crate::tests::GPG_IDENTITY]);

    if let ("init", Some(args)) = app.subcommand() {
      assert_eq!(super::init(args).is_ok(), true);
      assert_eq!(super::init(args).is_err(), true);

      let vault = wire::get_vault().expect("could not get vault metadata");

      assert_eq!(vault.get_identity(), "vault@apognu.github.com");

      return;
    }

    panic!("command init not triggering");
  }
}
