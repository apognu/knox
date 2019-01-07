# vault.rs

![](https://api.travis-ci.org/apognu/vault.rs.svg?branch=master)

An implementation of [apognu/vault](https://github.com/apognu/vault) in Rust with one big change: encryption is handled through GPG.

**Until this reaches 1.0.0, the Vault storage format is subject to breaking changes.**

## Summary

 * [Create the vault](#create-the-vault)
 * Secret management
   * [Add a secret](#add-a-secret)
     * [Confidentials attributes](#confidentials-attributes)
     * [File attribute](#file-attributes)
   * [Print a secret](#print-a-secret)
   * [Edit a secret](#edit-a-secret)
   * [Delete a secret](#delete-a-secret)

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