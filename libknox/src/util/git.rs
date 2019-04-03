use std::error::Error;

use git2::{
  Commit, Config, Cred, IndexAddOption, ObjectType, PushOptions, RemoteCallbacks, Repository,
  Signature,
};

use crate::{util::VaultError, VaultContext};

pub fn exists(vault: &VaultContext) -> bool {
  vault.has_pack(".git")
}

fn last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
  let object = repo.head()?.resolve()?.peel(ObjectType::Commit)?;

  object
    .into_commit()
    .map_err(|_| git2::Error::from_str("could not find latest commit"))
}

pub(crate) fn init(vault: &VaultContext) -> Result<(), Box<dyn Error>> {
  match Repository::init(&vault.path) {
    Ok(_) => commit(&vault, "Initialized knox repository."),
    Err(_) => Err(VaultError::throw("could not init git repository")),
  }
}

pub(crate) fn commit(vault: &VaultContext, message: &str) -> Result<(), Box<dyn Error>> {
  if !exists(&vault) {
    return Ok(());
  }

  match Repository::open(&vault.path) {
    Ok(repo) => {
      let (name, email) = Config::open_default()?
        .snapshot()
        .map(|c| {
          (
            c.get_string("user.name")
              .unwrap_or_else(|_| "Knox".to_string()),
            c.get_string("user.email")
              .unwrap_or_else(|_| "N/A".to_string()),
          )
        })
        .unwrap_or(("Knox".to_string(), "N/A".to_string()));

      let last_commit = last_commit(&repo).ok();
      let parent = match last_commit {
        Some(ref commit) => vec![commit],
        None => vec![],
      };

      let mut index = repo.index()?;
      index.add_all(&["*"], IndexAddOption::DEFAULT, None)?;

      let tree = repo.find_tree(index.write_tree()?)?;

      let author = Signature::now(&name, &email)?;

      repo.commit(Some("HEAD"), &author, &author, message, &tree, &parent)?;
      index.write()?;
    }
    Err(_) => {
      return Err(VaultError::throw(
        "could not open git repository, data was still written locally",
      ));
    }
  }

  Ok(())
}

pub(crate) fn set_origin(vault: &VaultContext, origin: &str) -> Result<(), Box<dyn Error>> {
  if !exists(&vault) {
    return Err(VaultError::throw("local git repository does not exist"));
  }

  let repo = Repository::open(&vault.path)?;

  repo.remote_set_url("origin", origin)?;

  Ok(())
}

pub(crate) fn push(vault: &VaultContext) -> Result<(), Box<dyn Error>> {
  if !exists(&vault) {
    return Err(VaultError::throw("local git repository does not exist"));
  }

  let repo = Repository::open(&vault.path)?;
  let mut remote = repo.find_remote("origin")?;
  let mut retry = false;
  let mut callbacks = RemoteCallbacks::new();

  callbacks.credentials(|_, user, credentials| {
    if retry {
      return Err(git2::Error::from_str(
        "ssh agent did not provide valid public key",
      ));
    }

    if credentials.contains(git2::CredentialType::USERNAME) {
      return Cred::username(user.unwrap_or("git"));
    }

    retry = true;

    Cred::ssh_key_from_agent("git")
  });

  let mut options = PushOptions::new();
  options.remote_callbacks(callbacks);

  remote.push(&["refs/heads/master:refs/heads/master"], Some(&mut options))?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use git2::{Repository, Sort};

  use knox_testing::spec;

  use crate::*;

  #[test]
  fn init() {
    let tmp = spec::setup();
    let context = VaultContext {
      path: tmp.path().to_str().unwrap().to_string(),
      vault: Vault::new(),
    };

    git::init(&context).expect("could not initialize git repository");

    let repo = Repository::open(tmp.path()).expect("could not open repository");
    let mut revwalk = repo.revwalk().expect("could not get revwalk");
    revwalk.push_head().expect("could not find HEAD");
    revwalk.set_sorting(Sort::REVERSE);

    let log: Vec<_> = revwalk.collect();

    assert_eq!(log.len(), 1);

    for rev in log {
      let commit = repo
        .find_commit(rev.unwrap())
        .expect("could not find root commit");

      assert_eq!(
        commit.summary().unwrap_or(""),
        "Initialized knox repository."
      );
    }
  }

  #[test]
  fn commit() {
    let tmp = spec::setup();
    let mut context = crate::spec::get_test_vault(tmp.path()).expect("could not get vault");
    context
      .git_init()
      .expect("could not create local git repository");

    context
      .write_entry("a", &Entry::default())
      .expect("could not write entry");
    context.commit("abcdef").expect("could not commit changes");

    let repo = Repository::open(tmp.path()).expect("could not open repository");
    let mut revwalk = repo.revwalk().expect("could not get revwalk");
    revwalk.push_head().expect("could not find HEAD");
    revwalk.set_sorting(Sort::REVERSE);

    for (i, rev) in revwalk.enumerate() {
      if i == 1 {
        let commit = repo
          .find_commit(rev.unwrap())
          .expect("could not find root commit");

        assert_eq!(commit.summary().unwrap_or(""), "abcdef");
      }
    }
  }

  #[test]
  fn set_remote() {
    let tmp = spec::setup();
    let context = crate::spec::get_test_vault(tmp.path()).expect("could not get vault");
    context
      .git_init()
      .expect("could not create local git repository");

    let repo = Repository::open(tmp.path()).expect("could not open repository");
    repo
      .remote("testremote", "https://git.example.com")
      .expect("could not set remote URL");

    let remote = repo
      .find_remote("testremote")
      .expect("could not find created remote");

    assert_eq!(remote.url().unwrap_or(""), "https://git.example.com");
  }
}
