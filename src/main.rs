mod utils;
mod cmdconfig;
mod blocks;
mod interpreter;

use clap::{Arg, App, SubCommand, crate_version, crate_authors};
use cmdconfig::handle_config;

fn main() {
    let matches = App::new("Piet Tools")
        .version(crate_version!())
        .author(crate_authors!())
        .about("A set of tools for the esoteric language Piet")
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Output with increased verbosity"))
        .subcommand(SubCommand::with_name("info")
            .arg(Arg::with_name("src")
                .help("Piet source image file")
                .index(1)
                .required(true))
            .arg(Arg::with_name("size")
                .long("size")
                .help("Width/Height of a codel, in pixels")
                .default_value("1"))
            .about("Show information about the Piet image file"))
        .subcommand(SubCommand::with_name("run")
            .arg(Arg::with_name("src")
                .help("Piet source image file")
                .index(1)
                .required(true))
            .arg(Arg::with_name("size")
                .long("size")
                .help("Width/Height of a codel, in pixels")
                .default_value("1"))
            .about("Interpret and run a Piet image file"))
        .get_matches();

    handle_config(matches);
}
