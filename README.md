# vault.rs

![](https://img.shields.io/travis/apognu/vault.rs/master.svg?style=flat-square)

An implementation of [apognu/vault](https://github.com/apognu/vault) in Rust with one big change: encryption is handled through GPG.

**Until this reaches 1.0.0, the Vault storage format is subject to breaking changes.**

## Summary

 * [Architecture](#architecture)
 * [Create the vault](#create-the-vault)
 * Secret management
   * [Add a secret](#add-a-secret)
     * [Confidentials attributes](#confidentials-attributes)
     * [File attribute](#file-attributes)
   * [Print a secret](#print-a-secret)
   * [Edit a secret](#edit-a-secret)
   * [Delete a secret](#delete-a-secret)
   * [Check if you've been pwned](#check-if-youve-been-pwned)

## Architecture

A vault is constituted of a __vault.meta_ file, at its root, containing the GPG identity used to encrypt the data as well as an index, mapping virtual secret paths to filesystem files. All filesystem paths in the vault are relative to this metadata file.

When a secret is created with a virtual file of _one/two/three_, a random UUID is generated, for instance, _2aef7bc6-856c-492d-aaee-07e0f2579812_, and the secret's attributes will be stored in a file named _2a/2aef7bc6-856c-492d-aaee-07e0f2579812_.

The mapping between virtual paths and filesystem paths is kept in the metadata file, and allows for retrieving data based on familiar user-defined paths. Hence, the metadata file is essential for using the vault and **should be backed up** along with the data. Secret files could still be manually decrypted and read, but you would lose the ability to refer to them through virtual paths.

The filesystem paths being random, and both the secret and metadata files being encrypted with your GPG public key, the filesystem does not give any information about what is stored inside the secrets.

All files are marshalled with _Protocol Buffers_ and encrypted through _gpg-agent_, producing armored ciphertext.

### Crates

This project is made of two distinct crates:

 * **vault**: A library containing all the logic of managing the vault. You could use this API to develop your own interface to your vaults.
 * **vault-bin**: A binary using the aforementioned library to provide a CLI interface to the vault.

## Create the vault

The following command creates an empty vault and takes the GPG identity for which the vault will be encrypted.

```
$ vault init myidentity@example.com
 INFO  vault::commands::init > vault initialized successfully
```

## Add a secret

```
$ vault add dir/subdir/website.com username=apognu password=Str0ngP@ss
 INFO  vault::commands::write > entry personal/website was successfully added to the vault
```

```vault``` is attribute-agnostic, there is no special handling of, for instance, the ```password``` attribute. You can add any number of attributes to an entry.

### Confidential attributes

One special kind of attribute is _confidential_ attributes. They only differ in that they are not printed on the console by default, and they are input interactively. Any attribute set without a value will trigger the prompt and will never be printed without the ```-p``` option.

```
$ vault add website.com username=apognu password=
Enter value for 'password': 
 INFO  vault::commands::write > entry personal/website was successfully added to the vault
```

### Generated passwords

One can generate random alphanumeric passwords with the attribute syntax ```attr=-```. By default, a random 16-character password will be generated for that attribute. Generated attributes will automatically be set as confidential.

The ```--symbols``` option adds special characters into the mix.

```
$ vault add personal/website username=apognu password=-
```

One can generate passwords with a different size with the ```-l``` / ```--length``` option.

### File attributes

An entire file can be embedded into an attribute with the syntax ```attr=@/path/to/file```. File attributes will never be printed on the console, and will require the use of ```-w``` to be used.

```
$ vault add personal/ssh pubkey=@/home/apognu/.ssh/id_rsa.pub privkey=@/home/apognu/.ssh/id_rsa
INFO  vault::commands::write > entry personal/ssh was successfully added to the vault
$ vault show personal/ssh
Store » ssh » keys
  privkey = <file content>
   pubkey = <file content>
```

## List secrets

```
$ vault list
🔒 Vault store:
  » one
  » two
  / subdir1
    / subdir2
      » secret1
      » secret2
      » secret3
      » secret4
  $ vault list subdir1/subdir2
🔒 Vault store:
  / subdir1
    / subdir2
      » secret1
      » secret2
      » secret3
      » secret4
```

## Print a secret

```
$ vault show dir/subdir/website.com
🔒 Vault store: / dir / subdir / website.com
   password = <redacted>
   username = apognu
        url = http://example.com/login
```

The ```-p``` option can be used to display the redacted attributes.

The ```-c``` option can be used to copy one attribute to the clipboard. By default, the attribute named ```password``` will be copied. If you would like to copy another attribute to your clipboard, use the ```-a``` option.

When you use the ```-w``` option in combination with showing a secret containing file attributes, all the file attributes of that secret will be written to files in a directory named after the secret path.

```
$ vault show my/secret/file
Store » my » secret » file
  file = <file content>
$ vault show -w my/secret/file
```

By default, all file attributes are written to matching files. If you wish to restrict which attribute gets considered for writing, use the ```-a``` option:

```
$ vault show -w -a file1 -afile2 my/secret/files
```

For file attributes, ```-s``` (for ```--stdout```) can also be used to print the content of a single attribute to your standard output.

```
$ vault show -w -a privkey -s sshkeys/corporate | ssh-add -
```

## Edit a secret

The syntax for modifying an existing secret is exactly the same as the one used to create one, with one addition: an optional list of attributes to delete.

```
$ vault edit website.com -d url username=newlogin password=
 INFO  vault::commands::write > entry website.com was successfully edited
```

This command will delete thre ```url``` attribute from the secret, change the ```username``` attribute to ```newlogin``` and prompt for the value of the redacted attribute ```password```

## Rename a secret

A secret can be renamed through the ```rename``` command:

```
$ vault rename my/first/secret new/location/secret
 INFO  vault::commands::write > entry my/first/secret was successfully renamed to new/location/secret
```

## Delete a secret

```
$ vault delete dir/subdir/website.com
 INFO  vault::commands::delete > entry 'dir/subdir/website.com' was successfully deleted from the vault
```

## Check if you've been pwned

Vault integrates Troy Hunt's [Have I Been Pwned](https://haveibeenpwned.com/) to check whether some of your passwords appear in a knowned data breach. For now, you can manually check every confidential attribute is a specific entry:

```
$ vault pwned my/super/password
INFO  vault::commands::pwned > Pwnage status for attributes at pwned/test
  ⚠ password -> PWNED
  ✓ secure -> CLEAR
  ⚠ apikey -> PWNED
```

In the future, we may implement a command to check your whole vault for breached secrets and check for pwnage at the moment of insertion.

# As a library

The library contained in ```vault``` can be used by your program to access and manipulate a vault (documentation pending). For example:

```
# Cargo.toml
# [dependencies]
# vault = { git = "https://github.com/apognu/vault.rs" }
#
# main.rs

use vault::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
  let handle = VaultHandle::open("/home/user/.vault")?;
  let entry = handle.read_entry("personal/websites/site-a")?;
  let attributes = entry
      .attributes
      .iter()
      .filter(|(_, attribute)| !attribute.confidential);

  for (key, attribute) in attributes {
      if let AttributeValue::String(value) = attribute.value() {
          println!("{} = {}", key, value);
      }
  }

  Ok(())
}
```