use glib::value::ToValue;
use gtk::prelude::*;
use libknox::*;
use relm::{Channel, Component, ContainerWidget, Sender};
use std::rc::Rc;
use std::sync::mpsc::{channel, Sender as MpscSender};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::Message as AppMessage;
use crate::vault;
use crate::widgets::attribute::AttributeRow;

enum TotpMessage {
    Tick,
    Refresh,
}

pub struct Model {
    relm: relm::Relm<Secret>,
    channel: Rc<Sender<AppMessage>>,
    item: Option<vault::Item>,
    attribute_children: Vec<Component<AttributeRow>>,
    totp_code: String,
    totp_expiration: f64,
    totp_killer: Option<MpscSender<()>>,
}

#[derive(Msg)]
pub enum Message {
    SetSecret(vault::Item),
    SetTotpExpiration(f64),
    RefreshTotp,
}

impl Secret {
    fn display_attributes(&mut self) {
        if let Some(item) = &self.model.item {
            for child in self.attributes.get_children() {
                self.model.attribute_children.clear();
                self.attributes.remove(&child);
            }
            if let Some(entry) = &item.entry {
                for (name, attr) in entry.get_attributes() {
                    let child = self.attributes.add_widget::<AttributeRow>((
                        Rc::clone(&self.model.channel),
                        name.clone(),
                        attr.clone(),
                    ));

                    self.model.attribute_children.push(child);
                }
            }
        }
    }

    fn display_totp(&mut self) {
        self.totp.hide();
        if let Some(item) = &self.model.item {
            if let Some(entry) = &item.entry {
                if entry.has_totp() {
                    if let Ok((totp, expiration)) = totp::get_totp(&entry, None) {
                        self.model.totp_code = totp;
                        self.model.totp_expiration = expiration as f64;
                        self.totp_code.set_text(&self.model.totp_code);

                        let r = self.model.relm.clone();
                        let exp = self.model.totp_expiration;

                        let (_, sender) = Channel::new(move |(event, expiration)| {
                            use TotpMessage::*;

                            match event {
                                Tick => r.stream().emit(Message::SetTotpExpiration(expiration)),
                                Refresh => r.stream().emit(Message::RefreshTotp),
                            }
                        });

                        let (killer, receiver) = channel();
                        self.model.totp_killer = Some(killer);

                        thread::spawn(move || loop {
                            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                            let epoch = now.as_secs_f64();
                            let min = exp - 30f64;
                            let value = (30f64 - (epoch - min)).floor();

                            if value >= 0.0 {
                                sender.send((TotpMessage::Tick, value)).unwrap();
                            } else {
                                sender.send((TotpMessage::Tick, 30.0)).unwrap();
                                sender.send((TotpMessage::Refresh, 0.0)).unwrap();
                                return;
                            }

                            if receiver.recv_timeout(Duration::from_secs(1)).is_ok() {
                                return;
                            }
                        });

                        self.totp_expiration
                            .set_property("max-value", &30f64.to_value())
                            .unwrap();

                        self.totp.show();
                    }
                }
            }
        }
    }
}

#[widget]
impl relm::Widget for Secret {
    type ModelParam = Rc<Sender<AppMessage>>;

    fn init_view(&mut self) {
        class!(self.title => "page-title");
        class!(self.path => "page-path");
        class!(self.attributes => "secret-attributes");
        class!(self.totp_title => "section-title");
        class!(self.totp_code => "totp-code");
    }

    fn model(relm: &relm::Relm<Self>, channel: Rc<Sender<AppMessage>>) -> Model {
        Model {
            relm: relm.clone(),
            channel,
            item: None,
            attribute_children: vec![],
            totp_code: String::new(),
            totp_expiration: 0.0,
            totp_killer: None,
        }
    }

    fn update(&mut self, event: Message) {
        use Message::*;
        match event {
            SetSecret(item) => {
                self.model.item = Some(item);
                self.model
                    .totp_killer
                    .as_ref()
                    .map(|sender| sender.send(()));
                self.model.totp_killer = None;

                self.display_attributes();
                self.display_totp();
            }

            SetTotpExpiration(expiration) => {
                self.totp_expiration
                    .set_property("value", &expiration.to_value())
                    .unwrap();
            }

            RefreshTotp => self.display_totp(),
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
                #[name="path"]
                gtk::Label {
                    label: &self.model.item.as_ref().map_or(String::new(), |item| item.path.clone()),
                    xalign: 0.0
                },
                #[name="title"]
                gtk::Label {
                    label: &self.model.item.as_ref().map_or(String::new(), |item| item.name.clone()),
                    xalign: 0.0
                },
                #[name="attributes"]
                gtk::Box {
                    orientation: gtk::Orientation::Vertical
                },
                #[name="totp"]
                gtk::Box {
                    orientation: gtk::Orientation::Vertical,
                    #[name="totp_title"]
                    gtk::Label {
                        text: "TOTP code",
                        xalign: 0.0
                    },
                    #[name="totp_code"]
                    gtk::Label {
                        text: &self.model.totp_code,
                    },
                    #[name="totp_expiration"]
                    gtk::LevelBar {}
                }
            }
        }
    }
}
