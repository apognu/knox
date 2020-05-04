use clipboard::{ClipboardContext, ClipboardProvider};
use gtk::prelude::*;
use libknox::*;
use relm::Sender;
use std::rc::Rc;

use crate::app::Message as AppMessage;

const REDACTED: &str = "REDACTED";

pub struct Model {
    relm: relm::Relm<AttributeRow>,
    channel: Rc<Sender<AppMessage>>,
    name: String,
    attribute: Attribute,
    revealed: bool,
    value: String,
}

#[derive(Msg)]
pub enum Message {
    SetValue(String),
    Reveal,
    Copy,
}

impl AttributeRow {
    fn normalize_value(&mut self) {
        if self.model.attribute.confidential {
            self.model.revealed = false;

            class!(self.value => "secret-attribute-confidential");

            self.model
                .relm
                .stream()
                .emit(Message::SetValue(REDACTED.to_string()));
        } else if self.model.attribute.file {
            self.reveal.set_sensitive(false);
            self.copy.set_sensitive(false);
            class!(self.value => "secret-attribute-file");

            self.model
                .relm
                .stream()
                .emit(Message::SetValue("FILE".to_string()));
        } else {
            self.reveal.set_sensitive(false);
            self.model
                .relm
                .stream()
                .emit(Message::SetValue(self.model.attribute.value.clone()));
        }
    }
}

#[widget]
impl relm::Widget for AttributeRow {
    type ModelParam = (Rc<Sender<AppMessage>>, String, Attribute);

    fn init_view(&mut self) {
        self.normalize_value();

        class!(self.attribute => "secret-attribute");
        class!(self.name => "secret-attribute-name");
    }

    fn model(
        relm: &relm::Relm<Self>,
        (channel, name, attribute): (Rc<Sender<AppMessage>>, String, Attribute),
    ) -> Model {
        Model {
            relm: relm.clone(),
            channel,
            name,
            attribute,
            revealed: false,
            value: String::new(),
        }
    }

    fn update(&mut self, event: Message) {
        use Message::*;

        match event {
            SetValue(value) => self.model.value = value,

            Reveal => {
                if self.model.revealed {
                    self.model.value = REDACTED.to_string();
                } else {
                    self.model.value = self.model.attribute.value.clone();
                }

                self.model.revealed = !self.model.revealed;
            }
            Copy => {
                let mut clip: ClipboardContext = ClipboardProvider::new().unwrap();
                clip.set_contents(self.model.attribute.value.clone())
                    .unwrap();

                self.model
                    .channel
                    .send(AppMessage::SetNotification(
                        "This attribute was copied to your clipboard".to_string(),
                    ))
                    .unwrap();
            }
        }
    }

    view! {
        #[name="attribute"]
        gtk::Box {
            orientation: gtk::Orientation::Horizontal,
            #[name="name"]
            gtk::Label {
                child: {
                    fill: true,
                    expand: true
                },
                text: &self.model.name,
                xalign: 0.0
            },
            #[name="value"]
            gtk::Label {
                text: &self.model.value
            },
            #[name="reveal"]
            gtk::Button {
                image: Some(&gtk::Image::new_from_pixbuf(Some(&load_pixbuf!("icons/reveal.svg", 16)))),
                clicked(_) => Message::Reveal
            },
            #[name="copy"]
            gtk::Button {
                image: Some(&gtk::Image::new_from_pixbuf(Some(&load_pixbuf!("icons/copy.svg", 16)))),
                clicked(_) => Message::Copy
            }
        }
    }
}
