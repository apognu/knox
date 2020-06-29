use std::error::Error;

use gpgme::data::IntoData;
use gpgme::{Context, Key, Protocol};

use crate::pb;
use crate::util::VaultError;

pub fn get_context() -> Result<Context, Box<dyn Error>> {
  let mut context = Context::from_protocol(Protocol::OpenPgp)?;
  context.set_armor(true);

  Ok(context)
}

pub fn get_keys(context: &mut Context, identities: &[String]) -> Result<Vec<Key>, Box<dyn Error>> {
  let keys: Vec<Key> = context.find_keys(identities)?.filter_map(Result::ok).filter(Key::can_encrypt).collect();

  if !keys.is_empty() {
    Ok(keys)
  } else {
    Err(VaultError::throw("no public key was found for provided identity"))
  }
}

pub fn encrypt(vault: &pb::Vault, object: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
  let mut context = get_context()?;
  let keys = get_keys(&mut context, vault.get_identities())?;

  if keys.len() < vault.get_identities().len() {
    return Err(VaultError::throw("could not retrieve the public keys for all provided identities"));
  }

  let mut output = Vec::new();
  context.encrypt(&keys, object, &mut output).map_err(|err| VaultError::throw(&err.description()))?;

  Ok(output)
}

pub fn decrypt<'a, T>(data: T) -> Result<Vec<u8>, Box<dyn Error>>
where
  T: IntoData<'a>,
{
  let mut output = Vec::new();
  get_context()?.decrypt(data, &mut output).map_err(|err| VaultError::throw(&err.description()))?;

  Ok(output)
}

#[cfg(test)]
mod tests {
  use knox_testing::spec;

  #[test]
  fn get_context() {
    assert_eq!(super::get_context().is_ok(), true);
  }

  #[test]
  fn get_keys() {
    spec::setup();
    let mut context = super::get_context().expect("could not get GPG context");

    assert_eq!(super::get_keys(&mut context, &spec::get_test_identities()).is_ok(), true);

    let keys = super::get_keys(&mut context, &spec::get_test_identities()).expect("could not get key");

    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].fingerprint(), Ok(spec::GPG_FINGERPRINT));

    assert_eq!(
      keys[0].user_ids().filter(|id| id.email() == Ok(spec::GPG_IDENTITY)).collect::<Vec<gpgme::keys::UserId>>().is_empty(),
      false
    );
  }

  #[test]
  fn encrypt_and_decrypt() {
    let tmp = spec::setup();
    let context = crate::spec::get_test_vault(tmp.path()).expect("could not get vault");

    let data = "foobarhelloworld".as_bytes();
    let ciphertext = super::encrypt(&context.vault, data).expect("could not encrypt data");

    assert_eq!(data.to_vec(), super::decrypt(ciphertext).expect("could not decrypt data"));
  }
}
