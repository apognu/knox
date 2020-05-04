use gtk::prelude::*;

pub struct Model {}

#[derive(Msg)]
pub enum Message {}

#[widget]
impl relm::Widget for Loader {
    fn init_view(&mut self) {
        self.spinner.start();
    }
    fn model(_relm: &relm::Relm<Self>, _: ()) -> Model {
        Model {}
    }

    fn update(&mut self, _event: Message) {}

    view! {
        #[name="spinner"]
        gtk::Spinner {
            child: {
                expand: true,
                fill: true
            },
            valign: gtk::Align::Center
        }
    }
}
