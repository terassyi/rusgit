extern crate clap;

use clap::{App, Arg, SubCommand};

use crate::cmd::init;
use crate::cmd::cat_file;

pub mod cmd;
mod object;

fn main() {
    // rusgit app definition
    let app = App::new("rusgit")
        .version("0.1.0")
        .about("mini git by rust")
        
        .subcommand(SubCommand::with_name("init")
            .about("Initialize rusgit repository."))
        .subcommand(SubCommand::with_name("cat-file")
            .about("cat git object file")
            .arg(Arg::with_name("hash")
                .help("hash key")
                .required(true))
            .arg(Arg::with_name("type")
                .help("show object type")
                .short("t")
                // .takes_value(true)
                )
            .arg(Arg::with_name("size")
                .help("show object size")
                .short("s")
                // .takes_value(true)
                )
            .arg(Arg::with_name("print")
                .help("pretty-print object's content")
                .short("p")
                // .takes_value(true)
                )
        );

    // parse subcommands and arguments
    let matches = app.get_matches();
    match matches.subcommand_matches("init") {
        Some(_) => {
            init::init_rusgit().unwrap();
            println!("Initialize rusgit repository!");
        },
        None => {}
    };
    match matches.subcommand_matches("cat-file") {
        Some(matches) => {
            let sha1 = matches.value_of("hash").unwrap();
            println!("hash key: {}", sha1);
            if let Some(_) = matches.args.get("type") {
                println!("show type");
                cat_file::cat_file(sha1, cat_file::CatFileType::Type).unwrap();
            }
            if let Some(_) = matches.args.get("size") {
                cat_file::cat_file(sha1, cat_file::CatFileType::Size).unwrap();
            }
            if let Some(_) = matches.args.get("print") {
                cat_file::cat_file(sha1, cat_file::CatFileType::Print).unwrap();
            }
        },
        None => {}
    }
}
