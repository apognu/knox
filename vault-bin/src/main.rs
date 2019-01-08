#![allow(renamed_and_removed_lints)]
#[macro_use]
extern crate clap;

mod commands;
mod util;

use log::*;
use std::error::Error;
use std::{env, process};

#[cfg(test)]
mod spec;

fn main() -> Result<(), Box<dyn Error>> {
    use clap::App;

    env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    let yml = load_yaml!("cli.yml");
    let app = App::from_yaml(yml).get_matches();

    let result = match app.subcommand() {
        ("init", Some(args)) => commands::init::init(args),
        ("list", Some(args)) => commands::display::list(args),
        ("show", Some(args)) => commands::display::show(args),
        ("add", Some(args)) => commands::write::add(args),
        ("edit", Some(args)) => commands::write::edit(args),
        ("rename", Some(args)) => commands::write::rename(args),
        ("delete", Some(args)) => commands::delete::delete(args),
        _ => {
            println!("{}", app.usage());
            process::exit(1);
        }
    };

    if let Err(error) = result {
        error!("{}", error.description());
        process::exit(1);
    }

    Ok(())
}
