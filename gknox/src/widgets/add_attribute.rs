use gtk::prelude::*;

pub struct Model {}

#[derive(Msg)]
pub enum Message {}

#[widget]
impl relm::Widget for AddAttribute {
  fn init_view(&mut self) {
    class!(self.attribute => "form-frame");
  }

  fn model(_relm: &relm::Relm<Self>, _: ()) -> Model {
    Model {}
  }

  fn update(&mut self, _event: Message) {}

  view! {
    gtk::Frame {
      #[name="attribute"]
      gtk::Box {
        orientation: gtk::Orientation::Horizontal,
        gtk::Entry {},
        gtk::Entry {},
      }
    }
  }
}
