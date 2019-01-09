use std::error::Error;

use gpgme::data::IntoData;
use gpgme::{Context, Key, Protocol};

use crate::pb;
use crate::util::VaultError;

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

  if !keys.is_empty() {
    Ok(keys)
  } else {
    Err(VaultError::throw(
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
  fn get_context() {
    assert_eq!(super::get_context().is_ok(), true);
  }

  #[test]
  fn get_keys() {
    let mut context = super::get_context().expect("could not get GPG context");

    assert_eq!(
      super::get_keys(&mut context, crate::spec::GPG_IDENTITY).is_ok(),
      true
    );

    let keys = super::get_keys(&mut context, crate::spec::GPG_IDENTITY).expect("could not get key");

    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].has_secret(), true);

    assert_eq!(
      keys[0].fingerprint(),
      Ok("AFD67570A1A7134F91D90EA7381C89FBA2E0D920")
    );

    assert_eq!(
      keys[0]
        .user_ids()
        .filter(|id| id.email() == Ok(crate::spec::GPG_IDENTITY))
        .collect::<Vec<gpgme::keys::UserId>>()
        .is_empty(),
      false
    );
  }

  #[test]
  fn encrypt_and_decrypt() {
    let tmp = crate::spec::setup();
    let handle = crate::spec::get_test_vault(tmp.path()).expect("could not get vault");

    let data = "foobarhelloworld".as_bytes();
    let ciphertext = super::encrypt(&handle.vault, data).expect("could not encrypt data");

    assert_eq!(
      data.to_vec(),
      super::decrypt(ciphertext).expect("could not decrypt data")
    );
  }
}
