use std::error::Error;
use std::thread;
use std::time::Duration;

use clipboard::{ClipboardContext, ClipboardProvider};
use colored::*;
use log::*;

use crate::pb;
use crate::util::{display, hierarchy, GenericError};
use crate::vault::{pack, wire};

pub(crate) fn list(_args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let vault = wire::get_vault()?;
  if vault.get_index().is_empty() {
    info!("the vault is empty");
    return Ok(());
  }

  let list = hierarchy::build(&vault);

  println!("🔒 Vault store:");
  hierarchy::print(&mut vec![], &list);

  Ok(())
}

pub(crate) fn show(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let vault = wire::get_vault()?;
  let path = args.value_of("path").unwrap();

  let print = args.is_present("print");
  let copy = args.is_present("copy");
  let write = args.is_present("write");

  if !vault.get_index().contains_key(path) {
    return Err(GenericError::throw("no entry was found at this path"));
  }

  let (_, real_path) = wire::hash_path(path, Some(vault.get_index().get(path).unwrap()));
  let entry: pb::Entry = pack::read(real_path)?;

  if copy {
    let name = args.value_of("attribute").unwrap_or("password");

    match entry.get_attributes().get(name) {
      Some(attribute) => {
        let mut clip: ClipboardContext = ClipboardProvider::new()?;
        clip.set_contents(attribute.value.clone())?;

        info!(
          "the content of the '{}' attribute was copied into your clipboard for 5 seconds",
          name.bold()
        );

        thread::sleep(Duration::from_secs(5));

        return Ok(());
      }
      None => {
        return Err(GenericError::throw(
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
            return Err(GenericError::throw(
              "the requested attribute does not exist in the entry",
            ))
          }
        },
        None => {
          return Err(GenericError::throw(
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
