use std::error::Error;
use std::fs::{remove_dir, remove_file, File, OpenOptions};
use std::io::Write;
use std::path::Path;

use protobuf::{parse_from_bytes, Message};

use crate::pb;
use crate::util::{normalize_path, GenericError};
use crate::vault::wire;

pub(crate) fn read<M, T>(path: T) -> Result<M, Box<dyn Error>>
where
  T: AsRef<Path>,
  M: Message,
{
  let pack = super::gpg::decrypt(&mut File::open(normalize_path(&path))?)?;
  let message = parse_from_bytes::<M>(&pack)?;

  Ok(message)
}

pub(crate) fn write<T>(vault: &pb::Vault, path: T, data: &[u8]) -> Result<(), Box<dyn Error>>
where
  T: AsRef<Path>,
{
  let mut file = OpenOptions::new()
    .create(true)
    .truncate(true)
    .write(true)
    .open(normalize_path(&path))?;

  file.write_all(&super::gpg::encrypt(&vault, data)?)?;

  Ok(())
}

pub(crate) fn delete<T>(vault: &mut pb::Vault, path: T) -> Result<(), Box<dyn Error>>
where
  T: AsRef<Path>,
{
  if let Some(entry) = vault
    .get_index()
    .get(&format!("{}", path.as_ref().display()))
  {
    wire::write_metadata(&vault)?;
    remove_file(normalize_path(&entry))?;

    for directory in Path::new(&entry).ancestors() {
      let _ = remove_dir(normalize_path(&format!("{}", directory.display())));
    }

    return Ok(());
  }

  Err(GenericError::throw(
    "requested entry does not exist in the vault",
  ))
}

pub(crate) fn exists<T>(path: T) -> bool
where
  T: AsRef<Path>,
{
  Path::new(&normalize_path(&path)).exists()
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::vault::{pack, wire};

  #[test]
  fn read_and_write_pack() {
    let _tmp = crate::tests::setup();
    let vault = crate::tests::get_test_vault();

    let wired = wire::pack(&vault).expect("could not create pack");
    pack::write(&vault, "pack.bin", &wired).expect("could not write pack");
    let retrieved: pb::Vault = super::read("pack.bin").expect("could not read pack");

    assert_eq!(retrieved, vault);
  }

  #[test]
  fn create_pack() {
    let message = pb::Vault {
      ..pb::Vault::default()
    };

    let wired = wire::pack(&message).expect("could not create pack");
    let retrieved: pb::Vault = super::parse_from_bytes(&wired).expect("could not read pack");

    assert_eq!(message, retrieved);
  }

  #[test]
  fn pack_exists() {
    let _tmp = crate::tests::setup();

    std::fs::File::create(normalize_path(&"test")).expect("could not create file");

    assert_eq!(true, super::exists("test"));
    assert_eq!(false, super::exists("foobar"));
  }
}
