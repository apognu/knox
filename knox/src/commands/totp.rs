use std::error::Error;

use base32::Alphabet::RFC4648;
use colored::*;
use libknox::{totp, TotpConfig_Hash, *};
use log::*;

use crate::util::vault_path;

pub(crate) fn configure(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let path = args.value_of("path").unwrap();

  let mut context = VaultContext::open(vault_path()?)?;

  if !context.vault.get_index().contains_key(path) {
    return Err(VaultError::throw("no entry was found at this path"));
  }

  let secret = args.value_of("secret").unwrap();

  let interval = args
    .value_of("interval")
    .unwrap_or("30")
    .parse::<u32>()
    .unwrap_or(30);

  let length = args
    .value_of("length")
    .unwrap_or("6")
    .parse::<u32>()
    .unwrap_or(6);

  let hash = match args.value_of("hash") {
    Some("sha1") => TotpConfig_Hash::SHA1,
    Some("sha256") => TotpConfig_Hash::SHA256,
    Some("sha512") => TotpConfig_Hash::SHA512,
    _ => TotpConfig_Hash::SHA1,
  };

  let mut entry = context.read_entry(&path)?;
  let has_totp = entry.has_totp();

  if !has_totp && !args.is_present("secret") {
    return Err(VaultError::throw(
      "you must provide the TOTP secret for a newly-created TOTP",
    ));
  }

  if !has_totp || args.is_present("secret") {
    let secret = base32::decode(RFC4648 { padding: false }, secret);

    if secret.is_none() {
      return Err(VaultError::throw(
        "the provided secret cannot be base32-decoded",
      ));
    } else {
      entry.mut_totp().set_secret(secret.unwrap());
    };
  }

  if !has_totp || args.is_present("interval") {
    entry.mut_totp().set_interval(u64::from(interval));
  }
  if !has_totp || args.is_present("length") {
    entry.mut_totp().set_length(length);
  }
  if !has_totp || args.is_present("hash") {
    entry.mut_totp().set_hash(hash);
  }

  context.write_entry(&path, &entry)?;

  info!(
    "the TOTP configuration for {} has been saved successfully",
    path.bold()
  );

  context.commit("Configure TOTP.")?;

  Ok(())
}

pub(crate) fn show(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let vault = VaultContext::open(vault_path()?)?;
  let path = args.value_of("path").unwrap();

  let entry = vault.read_entry(path)?;
  let totp = totp::get_totp(&entry, None)?;

  println!("{}", totp);

  Ok(())
}

#[cfg(test)]
mod tests {
  use clap::App;

  use knox_testing::spec;
  use libknox::*;

  #[test]
  fn configure() {
    let tmp = spec::setup();
    let mut vault = crate::spec::get_test_vault(tmp.path()).expect("could not write tests vault");

    let entry = Entry::default();

    vault
      .write_entry("foo/bar", &entry)
      .expect("could not write entry");

    let yml = load_yaml!("../cli.yml");
    let app = App::from_yaml(yml).get_matches_from(vec![
      "",
      "totp",
      "configure",
      "foo/bar",
      "--secret=abcdefghij",
    ]);

    if let ("totp", Some(args)) = app.subcommand() {
      if let ("configure", Some(args)) = args.subcommand() {
        assert_eq!(super::configure(args).is_ok(), true);

        let context = VaultContext::open(tmp.path()).expect("could not get vault");

        let entry = context
          .read_entry("foo/bar")
          .expect("could not read added entry");

        assert_eq!(entry.get_totp().get_interval(), 30);
        assert_eq!(entry.get_totp().get_length(), 6);
        assert_eq!(entry.get_totp().get_secret(), &[0, 68, 50, 20, 199, 66]);
        assert_eq!(entry.get_totp().get_hash(), TotpConfig_Hash::SHA1);
      }
    }

    let app = App::from_yaml(yml).get_matches_from(vec![
      "",
      "totp",
      "configure",
      "foo/bar",
      "--secret=poncfefgth",
      "--hash=sha512",
      "--length=10",
    ]);

    if let ("totp", Some(args)) = app.subcommand() {
      if let ("configure", Some(args)) = args.subcommand() {
        assert_eq!(super::configure(args).is_ok(), true);

        let context = VaultContext::open(tmp.path()).expect("could not get vault");

        let entry = context
          .read_entry("foo/bar")
          .expect("could not read added entry");

        assert_eq!(entry.get_totp().get_interval(), 30);
        assert_eq!(entry.get_totp().get_length(), 10);
        assert_eq!(
          entry.get_totp().get_secret(),
          &[123, 154, 34, 144, 166, 153]
        );
        assert_eq!(entry.get_totp().get_hash(), TotpConfig_Hash::SHA512);
      }
    }
  }
}
