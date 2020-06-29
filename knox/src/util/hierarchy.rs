use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use libknox::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Item {
  Directory(String, RefCell<Vec<Rc<Item>>>),
  File(String),
}

pub(crate) fn search(vault: &Vault, term: &str) -> Vec<String> {
  vault.get_index().keys().filter(|item| item.contains(term)).cloned().collect::<Vec<String>>()
}

pub(crate) fn build(paths: &Vault, prefix: Option<&str>) -> Option<Rc<Item>> {
  let root = Rc::new(Item::Directory("/".to_string(), RefCell::new(Vec::new())));
  let mut index: HashMap<String, Rc<Item>> = HashMap::new();

  let mut paths: Vec<Vec<&str>> = paths
    .get_index()
    .keys()
    .filter(|item| match prefix {
      None => true,
      Some(prefix) => item.starts_with(&format!("{}/", prefix)),
    })
    .map(|path| path.split('/').collect::<Vec<&str>>())
    .collect();

  if paths.is_empty() {
    return None;
  }

  paths.sort();

  for components in paths {
    let mut parent: Rc<Item> = Rc::clone(&root);
    let mut up_to_path: Vec<&str> = Vec::new();

    for (idx, component) in components.iter().enumerate() {
      up_to_path.push(component);

      if idx < (components.len() - 1) {
        let directory = index.entry(up_to_path.join("/")).or_insert_with(|| {
          let directory = Rc::new(Item::Directory((*component).to_string(), RefCell::new(Vec::new())));

          if let Item::Directory(_, inner) = &*parent {
            inner.borrow_mut().push(Rc::clone(&directory))
          }

          directory
        });

        parent = Rc::clone(&directory)
      } else {
        let file = Rc::new(Item::File((*component).to_string()));

        match &*parent {
          Item::Directory(_, inner) => inner.borrow_mut().push(Rc::clone(&file)),
          Item::File(_) => (),
        }

        parent = Rc::clone(&root);
      }
    }
  }

  Some(root)
}

pub(crate) fn print(path: &mut Vec<String>, item: &Rc<Item>) {
  use colored::*;

  match item.borrow() {
    Item::Directory(name, items) => {
      path.push(name.to_string());

      if name != "/" {
        println!("{: >width$} {}", "/".dimmed(), name.blue(), width = path.len() * 2);
      }

      for item in items.borrow().iter() {
        print(path, &item);
      }
    }

    Item::File(name) => {
      path.push(String::new());

      println!("{: >width$} {}", "»".bold(), name, width = path.len() * 2);
    }
  }

  path.pop();
}

#[cfg(test)]
mod tests {
  use std::cell::RefCell;
  use std::collections::HashMap;
  use std::rc::Rc;

  use super::Item;
  use libknox::*;

  #[test]
  fn build() {
    let data: HashMap<String, String> = [
      ("etc/hosts".to_string(), "".to_string()),
      ("etc/passwd".to_string(), "".to_string()),
      ("home/Documents/avatar.jpg".to_string(), "".to_string()),
      ("hello.txt".to_string(), "".to_string()),
    ]
    .iter()
    .cloned()
    .collect();

    let vault = Vault { index: data, ..Vault::default() };

    let expected = Rc::new(Item::Directory(
      "/".to_string(),
      RefCell::new(vec![
        Rc::new(Item::Directory(
          "etc".to_string(),
          RefCell::new(vec![Rc::new(Item::File("hosts".to_string())), Rc::new(Item::File("passwd".to_string()))]),
        )),
        Rc::new(Item::File("hello.txt".to_string())),
        Rc::new(Item::Directory(
          "home".to_string(),
          RefCell::new(vec![Rc::new(Item::Directory(
            "Documents".to_string(),
            RefCell::new(vec![Rc::new(Item::File("avatar.jpg".to_string()))]),
          ))]),
        )),
      ]),
    ));

    assert_eq!(Some(expected), super::build(&vault, None));
  }
}
