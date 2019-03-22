#![allow(renamed_and_removed_lints)]
#[macro_use]
extern crate clap;

mod commands;
mod util;

use std::error::Error;
use std::{env, process};

use clap::ArgMatches;
use log::*;

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
        ("info", Some(args)) => commands::info::info(args),
        ("identities", Some(args)) => match args.subcommand() {
            ("add", Some(args)) => commands::identities::add(args),
            ("delete", Some(args)) => commands::identities::delete(args),
            _ => usage(&app),
        },
        ("list", Some(args)) => commands::display::list(args),
        ("search", Some(args)) => commands::display::search(args),
        ("show", Some(args)) => commands::display::show(args),
        ("add", Some(args)) => commands::write::add(args),
        ("edit", Some(args)) => commands::write::edit(args),
        ("rename", Some(args)) => commands::write::rename(args),
        ("delete", Some(args)) => commands::delete::delete(args),
        ("pwned", Some(args)) => commands::pwned::pwned(args),
        _ => usage(&app),
    };

    if let Err(error) = result {
        error!("{}", error.description());
        process::exit(1);
    }

    Ok(())
}

fn usage(app: &ArgMatches) -> ! {
    println!("{}", app.usage());
    process::exit(1);
}
