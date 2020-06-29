use std::env;
use std::io::Write;

use gpgme::{edit, Context, Data, Protocol};
use tempfile::TempDir;

pub const GPG_FINGERPRINT: &str = "6A25FCF213C7779AD26DC50706CB643B42E7CD3E";
pub const GPG_IDENTITY: &str = "vault-test@apognu.github.com";

const GPG_PUBLIC_KEY: &str = "-----BEGIN PGP PUBLIC KEY BLOCK-----
mI0EXEiYGgEEAMnDE8fgdp6AKQy+/dVSdGkgMzpD0StplvXIK+W9coaDsfOmKDy0
b+rwL7/YXbj2Dht4kWQRg9YxBeBZPE0vjoR8KSoaeMp//EdKRSvJTeLYvvb4T/WY
2qfyUsE/+LSX4gHXTxk67ZQHF8WJmMNhOFOhwsTbeV2Xb5+ZujjH/EMjABEBAAG0
KVZhdWx0IFRlc3QgPHZhdWx0LXRlc3RAYXBvZ251LmdpdGh1Yi5jb20+iM4EEwEI
ADgWIQRqJfzyE8d3mtJtxQcGy2Q7QufNPgUCXEiYGgIbDQULCQgHAwUVCgkICwUW
AwIBAAIeAQIXgAAKCRAGy2Q7QufNPtYMA/9gwWwDlsWoDmvNYwnfe88saZynU229
WTt3EuHFO/pYyFsZrEUBbEYXp6e24O4Hn54XBiX8Z9hZrfi+rpgEfcIPB74Itt6j
AW0BEgqZRX6Vy6gjCKfbeaSP24qQTMDVPohMEzc6cg5dJ+P6SviXwChZWQ2Dh166
QdTCPerm6jDMQw==
=IADg
-----END PGP PUBLIC KEY BLOCK-----";

const GPG_SECRET_KEY: &str = "-----BEGIN PGP PRIVATE KEY BLOCK-----
lQHYBFxImBoBBADJwxPH4HaegCkMvv3VUnRpIDM6Q9EraZb1yCvlvXKGg7Hzpig8
tG/q8C+/2F249g4beJFkEYPWMQXgWTxNL46EfCkqGnjKf/xHSkUryU3i2L72+E/1
mNqn8lLBP/i0l+IB108ZOu2UBxfFiZjDYThTocLE23ldl2+fmbo4x/xDIwARAQAB
AAP9FR4ddmi4katxYHunHspUE+LCaeFKReR14ADVE2VKVOj42bs07/Gk2y7LmKVh
XegnHtn2QcaRiXw1FL/ST3PgUy7dasvtGlm/PHcwyeNSTMSgaqLpyxFE2aGDCrbR
680vELwR5YAanBkUYscAlExQPSOnUMC6/plqqHx71WRMMLkCANFfo5dEqw/1eOyu
FVUGdzilmr0S841s1JxLqiN7CHYpYY7ZUqcA1+BoqpCFldD1DvasXsl+k34WqegF
5w5EYekCAPaxgPBerS+9IKQw636ENlDXkrT/xAjIRofrNBPAu5jfMyvnrN0uR+jJ
3pexQaVduvDlricv5yM/Gapzxy/VqSsCAIxH+M7+PWLY4bFeA0DXxKuGnAC9g/KL
Usxyc8MxD+4B0VGk1OvPgB1/rxF65+XAOAh5z2y678aU6pV+L2/5Px6porQpVmF1
bHQgVGVzdCA8dmF1bHQtdGVzdEBhcG9nbnUuZ2l0aHViLmNvbT6IzgQTAQgAOBYh
BGol/PITx3ea0m3FBwbLZDtC580+BQJcSJgaAhsNBQsJCAcDBRUKCQgLBRYDAgEA
Ah4BAheAAAoJEAbLZDtC580+1gwD/2DBbAOWxagOa81jCd97zyxpnKdTbb1ZO3cS
4cU7+ljIWxmsRQFsRhenp7bg7gefnhcGJfxn2Fmt+L6umAR9wg8Hvgi23qMBbQES
CplFfpXLqCMIp9t5pI/bipBMwNU+iEwTNzpyDl0n4/pK+JfAKFlZDYOHXrpB1MI9
6ubqMMxD
=Z89s
-----END PGP PRIVATE KEY BLOCK-----";

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum EditorState {
  Start,
  Trust,
  Ultimate,
  Okay,
  Quit,
}

impl Default for EditorState {
  fn default() -> Self {
    EditorState::Start
  }
}

#[derive(Default)]
struct Editor;

impl edit::Editor for Editor {
  type State = EditorState;

  fn next_state(state: Result<Self::State, gpgme::Error>, status: edit::EditInteractionStatus, need_response: bool) -> Result<Self::State, gpgme::Error> {
    use self::EditorState as State;

    if !need_response {
      return state;
    }

    if status.args() == Ok(edit::PROMPT) {
      match state {
        Ok(State::Start) => Ok(State::Trust),
        Ok(State::Ultimate) => Ok(State::Quit),
        Ok(State::Okay) | Err(_) => Ok(State::Quit),
        Ok(State::Quit) => state,
        _ => Err(gpgme::Error::GENERAL),
      }
    } else if (status.args() == Ok("edit_ownertrust.value")) && (state == Ok(State::Trust)) {
      Ok(State::Ultimate)
    } else if (status.args() == Ok("edit_ownertrust.set_ultimate.okay")) && (state == Ok(State::Ultimate)) {
      Ok(State::Okay)
    } else {
      Err(gpgme::Error::GENERAL)
    }
  }

  fn action<W: Write>(&self, state: Self::State, mut out: W) -> Result<(), gpgme::Error> {
    use self::EditorState as State;

    match state {
      State::Trust => out.write_all(b"trust")?,
      State::Ultimate => out.write_all(b"5")?,
      State::Okay => out.write_all(b"y")?,
      State::Quit => write!(out, "{}", edit::QUIT)?,
      _ => return Err(gpgme::Error::GENERAL),
    }

    Ok(())
  }
}

pub fn setup() -> TempDir {
  let tmp = tempfile::tempdir().expect("could not create temporary directory");

  let mut context = Context::from_protocol(Protocol::OpenPgp).expect("could not create GPG context");
  context.set_armor(true);

  context
    .import(Data::from_bytes(&GPG_SECRET_KEY).expect("could not read GPG key"))
    .expect("could not import GPG secret key");

  context
    .import(Data::from_bytes(&GPG_PUBLIC_KEY).expect("could not read GPG key"))
    .expect("could not import GPG secret key");

  env::set_var("KNOX_PATH", tmp.path());

  let key = context.get_key(GPG_FINGERPRINT).expect("could not get GPG key");

  #[allow(deprecated)]
  context.edit_key_with(&key, Editor, &mut Vec::new()).expect("could not set key trust level");

  tmp
}

pub fn get_test_identities() -> Vec<String> {
  vec![GPG_FINGERPRINT.to_string()]
}
