use gtk::prelude::*;
use relm::{Component, ContainerWidget};

use crate::widgets::add_attribute::AddAttribute;

pub struct Model {
  attribute_children: Vec<Component<AddAttribute>>,
}

#[derive(Msg)]
pub enum Message {
  AddAttribute,
}

impl Add {
  fn add_attribute(&mut self) {
    self
      .model
      .attribute_children
      .push(self.attributes.add_widget::<AddAttribute>(()));
  }
}

#[widget]
impl relm::Widget for Add {
  fn init_view(&mut self) {
    class!(self.title => "page-title");
    class!(self.secret_title => "form-entry");
    class!(self.secret_path => "form-entry");
  }

  fn model(_relm: &relm::Relm<Self>, _: ()) -> Model {
    Model {
      attribute_children: vec![],
    }
  }

  fn update(&mut self, event: Message) {
    use Message::*;

    match event {
      AddAttribute => self.add_attribute(),
    }
  }

  view! {
    gtk::ScrolledWindow {
      child: {
          fill: true,
          expand: true
      },
      gtk::Box {
        orientation: gtk::Orientation::Vertical,
        #[name="title"]
        gtk::Label {
          text: "Add a secret",
          xalign: 0.0
        },
        gtk::Label {
          text: "Title",
          xalign: 0.0
        },
        #[name="secret_title"]
        gtk::Entry {},

        gtk::Label {
          text: "Path",
          xalign: 0.0
        },
        #[name="secret_path"]
        gtk::Entry {},
        gtk::Button {
          label: "Add an attribute",
          clicked(_) => Message::AddAttribute
        },
        #[name="attributes"]
        gtk::Box {
          orientation: gtk::Orientation::Vertical
        }
      }
    }
  }
}
