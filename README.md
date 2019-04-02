# knox

![](https://img.shields.io/travis/apognu/knox/master.svg?style=flat-square)

A structured secret manager encrypted through GPG.

**Until this reaches 1.0.0, the Vault storage format is subject to breaking changes.**

## Summary

 * [Architecture](#architecture)
 * [Installation](#installation)
 * [Create the vault](#create-the-vault)
 * Secret management
   * [Add a secret](#add-a-secret)
     * [Confidentials attributes](#confidentials-attributes)
     * [File attribute](#file-attributes)
   * [List secrets](#list-secrets)
   * [Search for secrets](#search-for-secrets)
   * [Print a secret](#print-a-secret)
   * [Edit a secret](#edit-a-secret)
   * [Delete a secret](#delete-a-secret)
   * [Check if you've been pwned](#check-if-youve-been-pwned)
   * [Manage identities](#manage-identities)
   * [Git integration](#git-integration)
   * [As a library](#as-a-library)

## Architecture

A vault is constituted of a _vault.meta file, at its root, containing the GPG identities used to encrypt the data as well as an index, mapping virtual secret paths to filesystem files. All filesystem paths in the vault are relative to this metadata file.

When a secret is created with a virtual path of one/two/three, a random UUID is generated, for instance, 2aef7bc6-856c-492d-aaee-07e0f2579812, and the secret's attributes will be stored in a file named 2a/2aef7bc6-856c-492d-aaee-07e0f2579812.

The mapping between virtual paths and filesystem paths is kept in the metadata file, and allows for retrieving data based on familiar user-defined paths. Hence, the metadata file is essential for using the vault and should be backed up along with the data. Secret files could still be manually decrypted and read, but you would lose the ability to refer to them through virtual paths.

The filesystem paths being random, and both the secret and metadata files being encrypted with your GPG public key, the filesystem does not give any information about what is stored inside the secrets.

All files are marshalled with Protocol Buffers and encrypted through gpg-agent, producing armored ciphertext.

When a vault is initialized, a local git repository is created in its directory, to record all operations.

### Crates

This project is made of two distinct crates:

 * **knox**: A library containing all the logic of managing the vault. You could use this API to develop your own interface to your vaults.
 * **knox-bin**: A binary using the aforementioned library to provide a CLI interface to the vault.

## Installation

Knox can be installed through `cargo`:

```
$ cargo install knox
```

As a library, you can add this stanza to your `Cargo.yaml`:

```
[dependencies]
libknox = "^0.1.1"
```

## Create the vault

The following command creates an empty vault and takes the GPG identity for which the vault will be encrypted.

```
$ knox init myidentity@example.com
 INFO  libknox::commands::init > vault initialized successfully
 INFO  knox::commands::init > local git repository initialized
```

By default, the vault will be created in ```$HOME/.knox```. You can change this path by setting the ```KNOX_PATH``` environment variable.

A local git repository will also be created in your vault directory (see [Git integration](#git-integration) for more information). This behavior can be disabled by passing `--no-git` to `init`.

## Add a secret

```
$ knox add dir/subdir/website.com username=apognu password=Str0ngP@ss
 INFO  libknox::commands::write > entry personal/website was successfully added to the vault
```

```vault``` is attribute-agnostic, there is no special handling of, for instance, the ```password``` attribute. You can add any number of attributes to an entry.

### Confidential attributes

One special kind of attribute is _confidential_ attributes. They only differ in that they are not printed on the console by default, and they are input interactively. Any attribute set without a value will trigger the prompt and will never be printed without the ```-p``` option.

```
$ knox add website.com username=apognu password=
Enter value for 'password': 
 INFO  libknox::commands::write > entry personal/website was successfully added to the vault
```

### Generated passwords

One can generate random alphanumeric passwords with the attribute syntax ```attr=-```. By default, a random 16-character password will be generated for that attribute. Generated attributes will automatically be set as confidential.

The ```--symbols``` option adds special characters into the mix.

```
$ knox add personal/website username=apognu password=-
```

One can generate passwords with a different size with the ```-l``` / ```--length``` option.

### File attributes

An entire file can be embedded into an attribute with the syntax ```attr=@/path/to/file```. File attributes will never be printed on the console, and will require the use of ```-w``` to be used.

```
$ knox add personal/ssh pubkey=@/home/apognu/.ssh/id_rsa.pub privkey=@/home/apognu/.ssh/id_rsa
INFO  libknox::commands::write > entry personal/ssh was successfully added to the vault
$ knox show personal/ssh
🔒 Knox » ssh » keys
  privkey = <file content>
   pubkey = <file content>
```

## List secrets

```
$ knox list
🔒 Knox
  » one
  » two
  / subdir1
    / subdir2
      » secret1
      » secret2
      » secret3
      » secret4
$ knox list subdir1/subdir2
🔒 Knox
  / subdir1
    / subdir2
      » secret1
      » secret2
      » secret3
      » secret4
```

You can filter the prefix for which to list secrets, for instance, `vault list subdir1/subdir2`.

## Search for secrets

You can search for secret matching a substring:

```
$ knox search social
🔒 Knox (search for social):
   » personal/social/facebook
   » personal/social/twitter
   » personal/social/linkedin
```

## Print a secret

```
$ knox show dir/subdir/website.com
🔒 Knox / dir / subdir / website.com
   password = <redacted>
   username = apognu
        url = http://example.com/login
```

The ```-p``` option can be used to display the redacted attributes.

The ```-c``` option can be used to copy one attribute to the clipboard. By default, the attribute named ```password``` will be copied. If you would like to copy another attribute to your clipboard, use the ```-a``` option.

When you use the ```-w``` option in combination with showing a secret containing file attributes, all the file attributes of that secret will be written to files in a directory named after the secret path.

```
$ knox show my/secret/file
🔒 Knox » my » secret » file
  file = <file content>
$ knox show -w my/secret/file
```

By default, all file attributes are written to matching files. If you wish to restrict which attribute gets considered for writing, use the ```-a``` option:

```
$ knox show -w -a file1 -a file2 my/secret/files
```

For file attributes, ```-s``` (for ```--stdout```) can also be used to print the content of a single attribute to your standard output.

```
$ knox show -w -a privkey -s sshkeys/corporate | ssh-add -
```

## Edit a secret

The syntax for modifying an existing secret is exactly the same as the one used to create one, with one addition: an optional list of attributes to delete.

```
$ knox edit website.com -d url username=newlogin password=
 INFO  libknox::commands::write > entry website.com was successfully edited
```

This command will delete thre ```url``` attribute from the secret, change the ```username``` attribute to ```newlogin``` and prompt for the value of the redacted attribute ```password```

## Rename a secret

A secret can be renamed through the ```rename``` command:

```
$ knox rename my/first/secret new/location/secret
 INFO  libknox::commands::write > entry my/first/secret was successfully renamed to new/location/secret
```

## Delete a secret

```
$ knox delete dir/subdir/website.com
 INFO  libknox::commands::delete > entry 'dir/subdir/website.com' was successfully deleted from the vault
```

## Check if you've been pwned

Vault integrates Troy Hunt's [Have I Been Pwned](https://haveibeenpwned.com/) to check whether some of your passwords appear in a known data breach. For now, you can manually check every confidential attribute is a specific entry:

```
$ knox pwned my/super/password
 INFO  libknox::commands::pwned > Pwnage status for attributes at pwned/test
 :: PWNED my/super/password:password
 :: PWNED my/super/password:secure
 :: PWNED my/super/password:apikey
```

The check is also performed for confidential attributes when adding or editing an entry. You can bypass this behavior by using the ```-f``` / ```--force``` flag on those commands.

You may also omit the ```PATH``` paramter to initiate a vault-wide check against the data breaches. This may take some time, but will check all confidential attributes in your vault:

```
INFO  libknox::commands::pwned > checking for pwned secret across your vault
 :: PWNED test/insecure/test1:password
 :: PWNED test/insecure/test1:apikey
 :: PWNED test/insecure/test2:password
█████████████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░ 53/70
[...]
 INFO  knox::commands::pwned > 5 secrets were found in HIBP's database
```

## Manage identities

When you initialize your vault, it is set up for one specific GPG identity. You may add and remove any GPG identity to your vault to allow other people to access it.

**Warning:** Check with your threat model if this is appropriate for you. When someone gets a copy of a vault encrypted with their GPG public key, there is no going back, they will be able to decrypt the content of that particular snapshot of the vault forever.

If you use a multi-identity vault, a single private key is sufficient to decrypt the vault, but **all** public keys are required to write to it.

When you add or remove an identity to or from the vault, all entries (including metadata) are reencrypted with the new set of public keys (as GPG recipients). This could take some time, depending on the size of your vault.

```
$ knox init myown@identity.com
 INFO  libknox::commands::init > vault initialized successfully at /vault
[...]
$ knox identities add myfriend@identity.com
 INFO  libknox::commands::identities > Writing metadata file...
 :: re-encrypting entry company/secret1
 :: re-encrypting entry personal/secret2
 :: re-encrypting entry company/secret2
 :: re-encrypting entry personal/secret1
 :: re-encrypting entry personal/secret3

$ knox identities delete myfriend@identity.com
```

## Git integration

Every time you edit your vault, either by adding, editing or deleting secrets, or changing identities, a git commit is created in your vault directory. **No identifying information** about your secret is ever stored in the commit messages, so as not to leak any insight into what you store in your vault.

You can manually set a remote and push your repository if you so desire:

```
$ knox git remote git@my.githost.com:passwords.git
 INFO  knox::commands::git > git remote URL set to 'git@my.githost.com:passwords.git'
$ knox git push
 INFO  knox::commands::git > vault modifications successfully pushed upstream
```

## As a library

The `examples` directory contain an example showing how to use `libknox` to manipulate vaults. You can run the example with:

```
$ cargo run --example simple
```