use std::error::Error;
use std::thread;
use std::time::Duration;

use clipboard::{ClipboardContext, ClipboardProvider};
use colored::*;
use log::*;

use crate::util::{display, hierarchy, vault_path};
use libknox::prelude::*;

pub(crate) fn list(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let path = args.value_of("path");
  let context = VaultContext::open(vault_path()?)?;
  if context.vault.get_index().is_empty() {
    info!("the vault is empty");
    return Ok(());
  }

  let list = hierarchy::build(&context.vault, path);

  match list {
    Some(list) => {
      println!("🔒 Vault store:");
      hierarchy::print(&mut vec![], &list);
    }
    None => {
      return Err(VaultError::throw(&format!(
        "the directory {} was not found in the vault",
        path.unwrap().bold()
      )));
    }
  }

  Ok(())
}

pub(crate) fn search(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let context = VaultContext::open(vault_path()?)?;
  let term = args.value_of("term").unwrap();

  let list = hierarchy::search(&context.vault, term);

  match list.len() {
    0 => info!("the term you searched for was not found in the vault"),
    _ => {
      println!("🔒 Vault store (search for {}):", term.dimmed());

      for path in list {
        println!(
          "   {} {}",
          "»".bold(),
          path.replace(term, &format!("{}", term.blue().bold()))
        );
      }
    }
  }

  Ok(())
}

pub(crate) fn show(args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
  let vault = VaultContext::open(vault_path()?)?;
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
        let value = match attribute.value() {
          AttributeValue::String(string) => string,
          AttributeValue::Binary(_) => {
            return Err(VaultError::throw(
              "attribute is binary, cannot copy to clipboard",
            ));
          }
        };

        let mut clip: ClipboardContext = ClipboardProvider::new()?;
        clip.set_contents(value)?;

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
        ));
      }
    }
  }

  if write {
    if args.is_present("stdout") {
      match args.value_of("attribute") {
        Some(attribute) => match entry.get_attributes().get(attribute) {
          Some(attribute) => match attribute.value() {
            AttributeValue::String(string) => println!("{}", string),
            AttributeValue::Binary(_) => {
              return Err(VaultError::throw(
                "attribute is binary, cannot print to console",
              ));
            }
          },
          None => {
            return Err(VaultError::throw(
              "the requested attribute does not exist in the entry",
            ));
          }
        },
        None => {
          return Err(VaultError::throw(
            "only a single attribute can be written to STDOUT, please use '--attribute'",
          ));
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
