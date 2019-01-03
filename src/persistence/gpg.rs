use std::error::Error;

use gpgme::data::IntoData;
use gpgme::{Context, Key, Protocol};

const DUMMY_IDENTITY: &str = "vault@appscho.com";

fn get_context() -> Result<Context, impl Error> {
  Context::from_protocol(Protocol::OpenPgp)
}

pub(crate) fn encrypt(object: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
  let mut context = get_context()?;
  let keys: Vec<Key> = context
    .find_keys(vec![DUMMY_IDENTITY])?
    .filter_map(|k| k.ok())
    .filter(|k| k.can_encrypt())
    .collect();

  let mut output = Vec::new();
  context.encrypt(&keys, object, &mut output)?;

  Ok(output)
}

pub(crate) fn decrypt<'a, T>(data: T) -> Result<Vec<u8>, Box<dyn Error>>
where
  T: IntoData<'a>,
{
  let mut output = Vec::new();
  get_context()?.decrypt(data, &mut output)?;

  Ok(output)
}
