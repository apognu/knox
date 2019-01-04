use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::pb;

#[derive(Debug)]
pub(crate) enum Item {
  Directory(String, RefCell<Vec<Rc<Item>>>),
  File(String),
}

pub(crate) fn build(paths: &pb::Vault) -> Rc<Item> {
  let root = Rc::new(Item::Directory("/".to_string(), RefCell::new(Vec::new())));
  let mut index: HashMap<String, Rc<Item>> = HashMap::new();

  let paths = paths
    .get_index()
    .keys()
    .map(|path| path.split('/').collect::<Vec<&str>>());

  for components in paths {
    let mut parent: Rc<Item> = Rc::clone(&root);
    let mut up_to_path: Vec<&str> = Vec::new();

    for (idx, component) in components.iter().enumerate() {
      up_to_path.push(component);

      if idx < (components.len() - 1) {
        let directory = index.entry(up_to_path.join("/")).or_insert_with(|| {
          let directory = Rc::new(Item::Directory(
            component.to_string(),
            RefCell::new(Vec::new()),
          ));

          if let Item::Directory(_, inner) = &*parent {
            inner.borrow_mut().push(Rc::clone(&directory))
          }

          directory
        });

        parent = Rc::clone(&directory)
      } else {
        let file = Rc::new(Item::File(component.to_string()));

        match &*parent {
          Item::Directory(_, inner) => inner.borrow_mut().push(Rc::clone(&file)),
          Item::File(_) => (),
        }

        parent = Rc::clone(&root);
      }
    }
  }

  root
}

pub(crate) fn print(path: &mut Vec<String>, item: &Rc<Item>) {
  use colored::*;

  match item.borrow() {
    Item::Directory(name, items) => {
      path.push(name.to_string());

      if name != "/" {
        for _ in 1..path.len() {
          print!("  ");
        }
        println!("{} {}", "/".dimmed(), name.blue());
      }

      for item in items.borrow().iter() {
        print(path, &item);
      }
    }

    Item::File(name) => {
      path.push(String::new());

      for _ in 1..path.len() {
        print!("  ");
      }
      println!("{} {}", "»".bold(), name);
    }
  }

  path.pop();
}
