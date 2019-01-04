use std::env;
use std::error::Error;
use std::fs::{create_dir_all, remove_dir, remove_file, File, OpenOptions};
use std::io::Write;
use std::path::Path;

use protobuf::{parse_from_bytes, CodedOutputStream, Message};

use crate::pb;
use crate::util::GenericError;

const BASE_PATH: &str = "/tmp/vault";
pub(crate) const METADATA_FILE: &str = "_vault.meta";

pub(crate) fn get_vault() -> Result<pb::Vault, Box<dyn Error>> {
  if !Path::new(&normalize_path(&METADATA_FILE)).exists() {
    return Err(GenericError::throw(&format!(
      "vault does not exist at {}, please initialize it",
      normalize_path(&""),
    )));
  }

  let pack = super::gpg::decrypt(&mut File::open(normalize_path(&METADATA_FILE))?)?;
  let vault = parse_from_bytes::<pb::Vault>(&pack)?;

  Ok(vault)
}

pub(crate) fn pack<M>(message: &M) -> Result<Vec<u8>, Box<dyn Error>>
where
  M: Message,
{
  let mut pack = Vec::new();
  let mut cos = CodedOutputStream::new(&mut pack);

  message.write_to(&mut cos)?;
  cos.flush()?;

  Ok(pack)
}

pub(crate) fn write_metadata(data: &[u8]) -> Result<(), Box<dyn Error>> {
  create_dir_all(normalize_path(&""))?;
  write_pack(METADATA_FILE, data)?;

  Ok(())
}

pub(crate) fn add_index(vault: &mut pb::Vault, path: &str, destination: &str) {
  vault
    .mut_index()
    .insert(path.to_string(), destination.to_string());
}

pub(crate) fn read_pack<M, T>(path: T) -> Result<M, Box<dyn Error>>
where
  T: AsRef<Path>,
  M: Message,
{
  let pack = super::gpg::decrypt(&mut File::open(normalize_path(&path))?)?;
  let message = parse_from_bytes::<M>(&pack)?;

  Ok(message)
}

pub(crate) fn write_pack<T>(path: T, data: &[u8]) -> Result<(), Box<dyn Error>>
where
  T: AsRef<Path>,
{
  let mut file = OpenOptions::new()
    .create(true)
    .truncate(true)
    .write(true)
    .open(normalize_path(&path))?;

  file.write_all(&data)?;

  Ok(())
}

pub(crate) fn delete_pack<T>(vault: &mut pb::Vault, path: T) -> Result<(), Box<dyn Error>>
where
  T: AsRef<Path>,
{
  if let Some(entry) = vault
    .mut_index()
    .remove(&format!("{}", path.as_ref().display()))
  {
    write_metadata(&super::gpg::encrypt(pack(vault)?)?)?;
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

pub(crate) fn pack_exists<T>(path: T) -> bool
where
  T: AsRef<Path>,
{
  Path::new(&normalize_path(&path)).exists()
}

fn normalize_path<T>(path: &T) -> String
where
  T: AsRef<Path>,
{
  let base = env::var("VAULT_PATH").unwrap_or_else(|_| BASE_PATH.to_string());

  format!("{}/{}", base, path.as_ref().display())
}

pub(crate) fn create_directories<T>(path: &T) -> Result<(), Box<dyn Error>>
where
  T: AsRef<Path>,
{
  let path = normalize_path(path);
  let mut path = path.split('/').collect::<Vec<&str>>();
  path.pop();

  create_dir_all(path.join("/"))?;

  Ok(())
}
