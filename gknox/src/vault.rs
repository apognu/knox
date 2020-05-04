use itertools::Itertools;
use libknox::*;
use std::cmp::Ordering;

#[derive(Clone, PartialEq, Eq)]
pub enum Kind {
    Folder,
    Secret,
}

impl Ord for Kind {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            return Ordering::Equal;
        } else if self == &Kind::Folder && other == &Kind::Secret {
            return Ordering::Less;
        }

        Ordering::Greater
    }
}

impl PartialOrd for Kind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone)]
pub struct Item {
    pub kind: Kind,
    pub path: String,
    pub name: String,
    pub entry: Option<Entry>,
}

pub fn filter_secrets(context: &VaultContext, path: &[String], term: &Option<String>) -> Vec<Item> {
    let prefix = path.join("/");
    let prefix = if prefix.is_empty() {
        prefix
    } else {
        format!("{}/", prefix)
    };
    context
        .vault
        .get_index()
        .iter()
        .filter(|(path, _)| {
            if let Some(term) = &term {
                path.contains(term)
            } else {
                path.starts_with(&prefix)
            }
        })
        .map(|(path, _)| {
            if let Some(_term) = &term {
                let tmp: Vec<&str> = path.split('/').collect();

                Item {
                    kind: Kind::Secret,
                    path: path.clone(),
                    name: (*tmp.iter().rev().next().unwrap()).to_string(),
                    entry: None,
                }
            } else {
                let mut tmp = path.trim_start_matches(&prefix).split('/');

                if tmp.clone().count() > 1 {
                    Item {
                        kind: Kind::Folder,
                        path: path.clone(),
                        name: tmp.next().unwrap().to_string(),
                        entry: None,
                    }
                } else {
                    Item {
                        kind: Kind::Secret,
                        path: path.clone(),
                        name: tmp.next().unwrap().to_string(),
                        entry: None,
                    }
                }
            }
        })
        .unique_by(|item| match term {
            Some(_) => (item.name.clone(), item.path.clone()),
            None => (item.name.clone(), String::new()),
        })
        .sorted_by_key(|item| (item.kind.clone(), item.name.clone()))
        .collect::<Vec<Item>>()
}
