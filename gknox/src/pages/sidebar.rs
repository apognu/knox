use gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use libknox::*;
use relm::Sender;
use std::rc::Rc;
use std::sync::Arc;

use crate::app::Message as AppMessage;
use crate::vault;

pub struct Model {
    channel: Rc<Sender<AppMessage>>,
    context: Arc<VaultContext>,
    prefix: Vec<String>,
    browser_model: gtk::ListStore,
    secrets: Vec<vault::Item>,
    term: Option<String>,
}

#[derive(Msg)]
pub enum Message {
    Search(String),
    Refresh,
    Refreshed(Arc<VaultContext>),
    BrowseUp,
    ItemSelected,
}

impl Sidebar {
    fn get_secrets(&mut self) {
        self.model.secrets =
            vault::filter_secrets(&self.model.context, &self.model.prefix, &self.model.term);
        self.model.browser_model.clear();

        for item in &self.model.secrets {
            match item.kind {
                vault::Kind::Folder => {
                    let pixbuf = load_pixbuf!("icons/folder.svg", 16);
                    self.model.browser_model.insert_with_values(
                        None,
                        &[0, 1],
                        &[&pixbuf, &format!("{}/", item.name)],
                    );
                }

                vault::Kind::Secret => {
                    let label = match self.model.term {
                        Some(_) => {
                            let mut path = item.path.split('/').collect::<Vec<&str>>();
                            path.remove(path.len() - 1);
                            format!(
                                "{}\n<span size=\"x-small\">{}</span>",
                                item.name,
                                path.join("/")
                            )
                        }
                        None => format!("{}", item.name),
                    };

                    let pixbuf = load_pixbuf!("icons/secret.svg", 16);
                    self.model
                        .browser_model
                        .insert_with_values(None, &[0, 1], &[&pixbuf, &label]);
                }
            }
        }
    }

    fn setup_browser(&mut self) {
        self.browser.set_model(Some(&self.model.browser_model));
        create_treeview_icon_column!(self, browser, "Icon", 0);
        create_treeview_markup_column!(self, browser, "Name", 1, true);

        self.get_secrets();
    }

    fn browse_up(&mut self) {
        self.model.prefix.pop();
        self.browse_up.set_sensitive(!self.model.prefix.is_empty());
        self.get_secrets();
    }

    fn on_item_selected(&mut self) {
        let (paths, _model) = self.browser.get_selection().get_selected_rows();
        let index = paths[0].get_indices()[0];
        let item = self.model.secrets[index as usize].clone();

        match item.kind {
            vault::Kind::Folder => {
                self.model.prefix.push(item.name);
                self.browse_up.set_sensitive(!self.model.prefix.is_empty());
                self.get_secrets();
            }

            vault::Kind::Secret => {
                self.model
                    .channel
                    .send(AppMessage::SecretSelected(item))
                    .unwrap();
            }
        }
    }
}

#[widget]
impl relm::Widget for Sidebar {
    type ModelParam = (Rc<Sender<AppMessage>>, Arc<VaultContext>);

    fn init_view(&mut self) {
        self.setup_browser();

        class!(self.sidebar => "sidebar");
        class!(self.actionbar => "actionbar");
        class!(self.search => "search");
    }

    fn model(
        _relm: &relm::Relm<Self>,
        (channel, context): (Rc<Sender<AppMessage>>, Arc<VaultContext>),
    ) -> Model {
        Model {
            channel,
            context,
            browser_model: gtk::ListStore::new(&[Pixbuf::static_type(), String::static_type()]),
            prefix: vec![],
            secrets: vec![],
            term: None,
        }
    }

    fn update(&mut self, event: Message) {
        use Message::*;

        match event {
            Search(term) => {
                self.model.term = if term.trim() == "" { None } else { Some(term) };

                match self.model.term {
                    Some(_) => self.browse_up.set_sensitive(false),
                    None => self.browse_up.set_sensitive(!self.model.prefix.is_empty()),
                }
                self.get_secrets();
            }

            Refresh => self.model.channel.send(AppMessage::Refresh).unwrap(),

            Refreshed(context) => {
                self.model.context = context;
                self.model.prefix = vec![];
                self.get_secrets();
            }

            BrowseUp => self.browse_up(),

            ItemSelected => self.on_item_selected(),
        }
    }

    view! {
        #[name="sidebar"]
        gtk::Box {
            orientation: gtk::Orientation::Vertical,
            #[name="search"]
            gtk::SearchEntry {
                child: {
                    pack_type: gtk::PackType::Start
                },
                search_changed(term) => Message::Search(term.get_text().unwrap().to_string())
            },
            #[name="actionbar"]
            gtk::ActionBar {
                #[name="browse_up"]
                gtk::Button {
                    sensitive: !self.model.prefix.is_empty(),
                    image: Some(&gtk::Image::new_from_pixbuf(Some(&load_pixbuf!("icons/up.svg", 16)))),
                    clicked(_) => Message::BrowseUp
                },
                gtk::Button {
                    image: Some(&gtk::Image::new_from_pixbuf(Some(&load_pixbuf!("icons/refresh.svg", 16)))),
                    clicked(_) => Message::Refresh
                }
            },
            gtk::ScrolledWindow {
                child: {
                    fill: true,
                    expand: true
                },
                min_content_width: 200,
                #[name="browser"]
                gtk::TreeView {
                    headers_visible: false,
                    activate_on_single_click: true,
                    row_activated(_, _, _) => Message::ItemSelected
                }
            },
        }
    }
}
