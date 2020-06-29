use std::env;

use libknox::gpg;
use libknox::*;

fn main() {
  let knox = VaultContext::open(env::var("KNOX_PATH").unwrap()).expect("FAIL");
  let mut knox_mut = VaultContext::open(env::var("KNOX_PATH").unwrap()).expect("FAIL");
  let mut context = gpg::get_context().unwrap();

  let identities = knox.vault.get_identities();

  for identity in identities {
    let keys = gpg::get_keys(&mut context, &[identity.clone()]).unwrap();

    for key in keys {
      knox_mut.remove_identity(identity);
      knox_mut.add_identity(key.fingerprint().unwrap());
    }
  }

  knox_mut.write().unwrap();
}
