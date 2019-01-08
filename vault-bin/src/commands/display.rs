use std::error::Error;
use std::thread;
use std::time::Duration;

use clipboard::{ClipboardContext, ClipboardProvider};
use colored::*;
use log::*;

use crate::util::{display, hierarchy, vault_path};
use vault::prelude::*;

pub(crate) fn list(_args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let handle = VaultHandle::open(vault_path())?;
  if handle.vault.get_index().is_empty() {
    info!("the vault is empty");
    return Ok(());
  }

  let list = hierarchy::build(&handle.vault);

  println!("🔒 Vault store:");
  hierarchy::print(&mut vec![], &list);

  Ok(())
}

pub(crate) fn show(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let vault = VaultHandle::open(vault_path())?;
  let path = args.value_of("path").unwrap();

  let print = args.is_present("print");
  let copy = args.is_present("copy");
  let write = args.is_present("write");

  let entry = vault.read_entry(path)?;

  if copy {
    let name = if let Some(attributes) = args.values_of("attribute") {
      attributes.collect()
    } else {
      vec!["password"]
    };

    if name.len() > 1 {
      return Err(VaultError::throw(
        "only one attribute can be copied to the clipboard",
      ));
    }

    match entry.get_attributes().get(name[0]) {
      Some(attribute) => {
        let mut clip: ClipboardContext = ClipboardProvider::new()?;
        clip.set_contents(attribute.value.clone())?;

        info!(
          "the content of the '{}' attribute was copied into your clipboard for 5 seconds",
          name[0].bold()
        );

        thread::sleep(Duration::from_secs(5));

        return Ok(());
      }
      None => {
        return Err(VaultError::throw(
          "the requested attribute does not exist in the entry",
        ))
      }
    }
  }

  if write {
    if args.is_present("stdout") {
      match args.value_of("attribute") {
        Some(attribute) => match entry.get_attributes().get(attribute) {
          Some(attribute) => println!("{}", display::get_attribute_value(attribute)),
          None => {
            return Err(VaultError::throw(
              "the requested attribute does not exist in the entry",
            ))
          }
        },
        None => {
          return Err(VaultError::throw(
            "only a single attribute can be written to STDOUT, please use '--attribute'",
          ))
        }
      }

      return Ok(());
    }

    display::write_files(
      path,
      &entry,
      &args
        .value_of("attribute")
        .and_then(|attribute| Some(attribute.split(',').collect::<Vec<&str>>())),
    )?;

    return Ok(());
  }

  display::entry(path, &entry, print);

  Ok(())
}
