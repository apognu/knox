use gtk::prelude::*;
use libknox::*;
use relm::{Channel, Sender};
use std::process;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::assets::Asset;
use crate::pages::{
    add::Add,
    header::Header,
    loader::Loader,
    secret::{Message as SecretMessage, Secret},
    sidebar::{Message as SidebarMessage, Sidebar},
};
use crate::vault;

pub struct Model {
    path: String,
    relm: relm::Relm<App>,
    channel: Rc<Sender<Message>>,
    context: Arc<VaultContext>,
    notification: String,
}

#[derive(Msg)]
pub enum Message {
    ShowAdd,
    SetNotification(String),
    HideNotification,
    Refresh,
    Refreshed(VaultContext),
    SecretSelected(vault::Item),
    SecretDecrypted(vault::Item),
    Quit,
}

impl App {
    fn load_style(&self) {
        let screen = self.window.get_screen().unwrap();
        let css = gtk::CssProvider::new();
        let stylesheet = Asset::get("style.css").unwrap();

        css.load_from_data(&stylesheet).unwrap();

        gtk::StyleContext::add_provider_for_screen(&screen, &css, 0);
    }

    fn refresh(&mut self) {
        let path = self.model.path.clone();

        run_on_thread!(self => Message::Refreshed(VaultContext) {
            VaultContext::open(&path).ok()
        });
    }
}

#[widget]
#[allow(clippy::cognitive_complexity)]
impl relm::Widget for App {
    type ModelParam = String;

    fn init_view(&mut self) {
        self.load_style();
        self.stack.show_all();
        self.notification.hide();

        class!(self.content => "content");
        class!(self.add => "content");
    }
    fn model(relm: &relm::Relm<Self>, path: String) -> Model {
        let context = VaultContext::open(&path);

        if context.is_err() {
            process::exit(1);
        }

        let context = context.unwrap();
        let r = relm.clone();

        let (_, sender) = Channel::new(move |event| {
            r.stream().emit(event);
        });
        Model {
            relm: relm.clone(),
            path,
            channel: Rc::new(sender),
            context: Arc::new(context),
            notification: String::new(),
        }
    }

    fn update(&mut self, event: Message) {
        use Message::*;

        match event {
            ShowAdd => self.stack.set_visible_child(&self.add),

            SetNotification(message) => {
                self.model.notification = message;
                self.notification.set_revealed(true);
                self.notification.show();

                run_on_thread!(self => Message::HideNotification {
                    thread::sleep(Duration::from_secs(2));

                    Some(())
                });
            }

            HideNotification => self.notification.hide(),

            Refresh => self.refresh(),

            Refreshed(context) => {
                self.model.context = Arc::new(context);
                self.sidebar
                    .emit(SidebarMessage::Refreshed(Arc::clone(&self.model.context)));
            }

            SecretSelected(item) => {
                self.stack.set_visible_child(&self.loader);

                let context = Arc::clone(&self.model.context);

                run_on_thread!(self => Message::SecretDecrypted(vault::Item) {
                    context
                        .read_entry(&item.path)
                        .and_then(|entry| Ok(vault::Item { entry: Some(entry), ..item }))
                        .ok()
                });
            }

            SecretDecrypted(item) => {
                self.secret.emit(SecretMessage::SetSecret(item));
                self.stack.set_visible_child(&self.content);
            }

            Quit => gtk::main_quit(),
        }
    }

    view! {
        #[name="window"]
        gtk::ApplicationWindow {
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                Header(Rc::clone(&self.model.channel)),
                gtk::Box {
                    child: {
                        fill: true,
                        expand: true
                    },
                    orientation: gtk::Orientation::Horizontal,
                    #[name="sidebar"]
                    Sidebar((Rc::clone(&self.model.channel), Arc::clone(&self.model.context))),
                    #[name="stack"]
                    gtk::Stack {
                        child: {
                            fill: true,
                            expand: true
                        },
                        #[name="idle"]
                        gtk::Box {
                            orientation: gtk::Orientation::Vertical,
                            gtk::Label {
                                child: {
                                    expand: true
                                },
                                text: "Select a secret to access its attributes"
                            }
                        },
                        #[name="loader"]
                        gtk::Box {
                            orientation: gtk::Orientation::Vertical,
                            Loader {}
                        },
                        #[name="content"]
                        gtk::Box {
                            orientation: gtk::Orientation::Vertical,
                            #[name="secret"]
                            Secret(Rc::clone(&self.model.channel)) {}
                        },
                        #[name="add"]
                        gtk::Box {
                            orientation: gtk::Orientation::Vertical,
                            Add
                        }
                    }
                },
                #[name="notification"]
                gtk::InfoBar {
                    message_type: gtk::MessageType::Info,
                    gtk::Label {
                        text: &self.model.notification
                    }
                },
            },
            delete_event(_, _) => (Message::Quit, gtk::Inhibit(false))
        }
    }
}
