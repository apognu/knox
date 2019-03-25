#![allow(renamed_and_removed_lints)]

//! # Knox
//!
//! Knox is a secret vault (aka password manager) encrypted with GPG keys.
//! Libknox allows you to manipulate vaults at a low level.
//!
//! # Architecture
//!
//! A vault is constituted of a __vault.meta_ file, at its root, containing the
//! GPG identities used to encrypt the data as well as an index, mapping
//! _virtual secret paths_ to filesystem files. All filesystem paths in the
//! vault are relative to this metadata file.
//!
//! When a secret is created with a virtual path of _one/two/three_, a random
//! UUID is generated, for instance, _2aef7bc6-856c-492d-aaee-07e0f2579812_,
//! and the secret's attributes will be stored in a file named
//! _2a/2aef7bc6-856c-492d-aaee-07e0f2579812_.
//!
//! The mapping between virtual paths and filesystem paths is kept in the
//! metadata file, and allows for retrieving data based on familiar
//! user-defined paths. Hence, the metadata file is essential for using the
//! vault and **should be backed up** along with the data. Secret files could
//! still be manually decrypted and read, but you would lose the ability to
//! refer to them through virtual paths.
//!
//! The filesystem paths being random, and both the secret and metadata files
//! being encrypted with your GPG public key, the filesystem does not give any
//! information about what is stored inside the secrets.
//!
//! All files are marshalled with _Protocol Buffers_ and encrypted through
//! _gpg-agent_, producing armored ciphertext.
//!
//! # Example
//!
//! This example below shows how to use the libknox API to create and
//! manipulate a vault. It assumes the `/tmp/knox-example` is empty and that
//! that your GPG agent has keys with the `vault-test@apognu.github.com`
//! identity.
//!
//! It can be run with `cargo run --example simple`.
//!
//! ```
//! use libknox::*;
//!
//! fn main() {
//!   // Create a new vault with the given GPG identity
//!   let id = vec!["vault-test@apognu.github.com".to_string()];
//!   let mut vault = VaultContext::create("/tmp/knox-example", &id).expect("FAIL");
//!
//!   // Create a new entry with three attributes
//!   let mut entry = Entry::new();
//!   entry.add_attribute("username", "bob");
//!   entry.add_confidential_attribute("password", "foobar");
//!   entry.add_confidential_attribute(
//!     "apikey",
//!     "3OJL07P+W5zODH2J1Wv7rXh5i9UpR0mpvPW7ygIMih82J8P95krJZXyERqbi/XS",
//!   );
//!
//!   // Write the entry and the metadata pointing to it
//!   vault
//!     .write_entry("personal/website.com", &entry)
//!     .expect("FAIL");
//!
//!   // Open the prevously created vault and read the written entry
//!   let vault = VaultContext::open("/tmp/knox-example").expect("FAIL");
//!   let entry = vault.read_entry("personal/website.com").expect("FAIL");
//!
//!   // Loop over the attributes and print them
//!   for (key, attribute) in entry.get_attributes() {
//!     if attribute.confidential {
//!       println!("{} = {} (CONFIDENTIAL)", key, attribute.value);
//!     } else {
//!       println!("{} = {}", key, attribute.value);
//!     }
//!   }
//! }
//! ```

mod gpg;
mod pb;
mod util;

mod vault;

pub use vault::*;

#[cfg(test)]
mod spec;
