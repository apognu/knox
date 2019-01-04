use std::error::Error;
use std::fmt;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::Write;
use std::path::Path;

use protobuf::{parse_from_bytes, CodedOutputStream, Message};

use crate::pb;

const BASE_PATH: &str = "/tmp/vault";
pub(crate) const METADATA_FILE: &str = "_vault.meta";

pub(crate) fn get_vault() -> Result<pb::Vault, Box<dyn Error>> {
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
  T: AsRef<Path> + fmt::Display,
  M: Message,
{
  let pack = super::gpg::decrypt(&mut File::open(normalize_path(&path))?)?;
  let message = parse_from_bytes::<M>(&pack)?;

  Ok(message)
}

pub(crate) fn write_pack<T>(path: T, data: &[u8]) -> Result<(), Box<dyn Error>>
where
  T: AsRef<Path> + fmt::Display,
{
  let mut file = OpenOptions::new()
    .create(true)
    .truncate(true)
    .write(true)
    .open(normalize_path(&path))?;

  file.write_all(&data)?;

  Ok(())
}

pub(crate) fn pack_exists<T>(path: T) -> bool
where
  T: AsRef<Path> + fmt::Display,
{
  Path::new(&normalize_path(&path)).exists()
}

fn normalize_path<T>(path: &T) -> String
where
  T: AsRef<Path> + fmt::Display,
{
  format!("{}/{}", BASE_PATH, path)
}

pub(crate) fn create_directories<T>(path: &T) -> Result<(), Box<dyn Error>>
where
  T: AsRef<Path> + fmt::Display,
{
  let path = normalize_path(path);
  let mut path = path.split('/').collect::<Vec<&str>>();
  path.pop();

  create_dir_all(path.join("/"))?;

  Ok(())
}
