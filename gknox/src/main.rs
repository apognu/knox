#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate rust_embed;

#[macro_use]
mod macros;
mod app;
#[macro_use]
mod assets;
mod pages;
mod vault;
mod widgets;

use relm::Widget;
use std::env;
use std::error::Error;
use std::process;

use crate::app::App;

fn main() -> Result<(), Box<dyn Error>> {
    if let Some(path) = env::args().nth(1) {
        App::run(path).expect("could not start application");
    } else if let Some(home) = dirs::home_dir() {
        App::run(format!("{}/.knox", home.display())).expect("could not start application");
    } else {
        process::exit(1);
    }

    Ok(())
}
