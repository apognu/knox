use std::error::Error;

use gpgme::data::IntoData;
use gpgme::{Context, Key, Protocol};

use crate::pb;
use crate::util::GenericError;

pub(crate) fn get_context() -> Result<Context, Box<dyn Error>> {
  let mut context = Context::from_protocol(Protocol::OpenPgp)?;
  context.set_armor(true);

  Ok(context)
}

pub(crate) fn get_keys(context: &mut Context, identity: &str) -> Result<Vec<Key>, Box<dyn Error>> {
  let keys: Vec<Key> = context
    .find_secret_keys(vec![identity])?
    .filter_map(|k| k.ok())
    .filter(|k| k.can_encrypt())
    .collect();

  if keys.len() > 0 {
    Ok(keys)
  } else {
    Err(GenericError::throw(
      "no private key was found for provided identity",
    ))
  }
}

pub(crate) fn encrypt(vault: &pb::Vault, object: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
  let mut context = get_context()?;
  let keys = get_keys(&mut context, vault.get_identity())?;

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

#[cfg(test)]
mod tests {
  #[test]
  fn encrypt_and_decrypt() {
    let vault = crate::tests::get_test_vault();
    let data = "foobarhelloworld".as_bytes();
    let ciphertext = super::encrypt(&vault, data).expect("could not encrypt data");

    assert_eq!(
      data.to_vec(),
      super::decrypt(ciphertext).expect("could not decrypt data")
    );
  }
}
