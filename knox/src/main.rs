#![allow(renamed_and_removed_lints)]
#[macro_use]
extern crate clap;

mod commands;
mod util;

use std::error::Error;
use std::{env, process};

use clap::App;
use log::*;

#[cfg(test)]
mod spec;

fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    let yml = load_yaml!("cli.yml");
    let mut app = App::from_yaml(yml)
        .name(crate_name!())
        .version(crate_version!())
        .author(crate_authors!());

    let matches = app.clone().get_matches();

    let result = match matches.subcommand() {
        ("init", Some(args)) => commands::init::init(args),
        ("info", Some(args)) => commands::info::info(args),
        ("identities", Some(args)) => match args.subcommand() {
            ("add", Some(args)) => commands::identities::add(args),
            ("delete", Some(args)) => commands::identities::delete(args),
            _ => usage(&mut app),
        },
        ("list", Some(args)) => commands::display::list(args),
        ("search", Some(args)) => commands::display::search(args),
        ("show", Some(args)) => commands::display::show(args),
        ("add", Some(args)) => commands::write::add(args),
        ("edit", Some(args)) => commands::write::edit(args),
        ("totp", Some(args)) => match args.subcommand() {
            ("configure", Some(args)) => commands::totp::configure(args),
            ("show", Some(args)) => commands::totp::show(args),
            _ => usage(&mut app),
        },
        ("rename", Some(args)) => commands::write::rename(args),
        ("delete", Some(args)) => commands::delete::delete(args),
        ("pwned", Some(args)) => commands::pwned::pwned(args),
        ("git", Some(args)) => match args.subcommand() {
            ("remote", Some(args)) => commands::git::set_remote(args),
            ("push", Some(args)) => commands::git::push(args),
            _ => usage(&mut app),
        },
        _ => usage(&mut app),
    };

    if let Err(error) = result {
        error!("{}", error.description());
        process::exit(1);
    }

    Ok(())
}

fn usage(app: &mut App) -> ! {
    let _ = app.print_help();
    process::exit(1);
}
