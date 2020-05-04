use gtk::prelude::*;
use relm::Sender;
use std::rc::Rc;

use crate::app::Message as AppMessage;

pub struct Model {
    relm: relm::Relm<Header>,
    channel: Rc<Sender<AppMessage>>,
    settings: gtk::Popover,
    settings_about: gtk::Button,
}

#[derive(Msg)]
pub enum Message {
    ShowSettings,
    ShowAbout,
    ShowAdd,
}

impl Header {
    fn setup_settings(&mut self) {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

        self.model.settings_about.set_label("About...");
        relm::connect!(
            self.model.relm,
            self.model.settings_about,
            connect_clicked(_),
            Message::ShowAbout
        );
        vbox.add(&self.model.settings_about);

        self.model.settings.set_relative_to(Some(&self.settings));
        self.model.settings.add(&vbox);
        self.model.settings.hide();
    }
}

#[widget]
impl relm::Widget for Header {
    type ModelParam = Rc<Sender<AppMessage>>;

    fn init_view(&mut self) {
        self.setup_settings();
    }
    fn model(relm: &relm::Relm<Self>, channel: Rc<Sender<AppMessage>>) -> Model {
        Model {
            relm: relm.clone(),
            channel,
            settings: gtk::Popover::new(None::<&gtk::Button>),
            settings_about: gtk::Button::new(),
        }
    }

    fn update(&mut self, event: Message) {
        use Message::*;

        match event {
            ShowSettings => self.model.settings.show_all(),

            ShowAbout => {
                let dialog = gtk::AboutDialog::new();
                dialog.set_program_name("knox");
                dialog.set_version(Some(&std::env::var("CARGO_PKG_VERSION").unwrap()));
                dialog.set_authors(&["Antoine POPINEAU"]);
                dialog.set_website(Some("https://github.com/apognu/knox"));
                dialog.set_license_type(gtk::License::MitX11);
                dialog.show();
            }

            ShowAdd => self.model.channel.send(AppMessage::ShowAdd).unwrap(),
        }
    }

    view! {
        gtk::HeaderBar {
            title: Some("Knox"),
            show_close_button: true,
            #[name="add"]
            gtk::Button {
                child: {
                    pack_type: gtk::PackType::Start
                },
                image: Some(&gtk::Image::new_from_pixbuf(Some(&load_pixbuf!("icons/add.svg", 16)))),
                clicked(_) => Message::ShowAdd
            },
            #[name="settings"]
            gtk::Button {
                child: {
                    pack_type: gtk::PackType::End
                },
                image: Some(&gtk::Image::new_from_pixbuf(Some(&load_pixbuf!("icons/settings.svg", 16)))),
                clicked(_) => Message::ShowSettings
            }
        }
    }
}
