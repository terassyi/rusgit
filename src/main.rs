extern crate clap;

use clap::{App, Arg, SubCommand};

use crate::cmd::init;
use crate::cmd::cat_file;
use crate::cmd::hash_object;
use crate::cmd::update_index;
use crate::cmd::ls_files;

pub mod cmd;
mod object;
mod index;

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
                ))
        .subcommand(SubCommand::with_name("hash-object")
            .about("hash object")
            .arg(Arg::with_name("file")
                .help("file path hashing")
                .required(true))
            .arg(Arg::with_name("write")
                .help("write the object into the object database")
                .short("w")))
        .subcommand(SubCommand::with_name("update-index")
            .about("update index")
            .arg(Arg::with_name("add")
                .help("do not ignore new filesr")
                .long("add"))
        )
        .subcommand(SubCommand::with_name("ls-files")
            .about("list up files")
            .arg(Arg::with_name("stage")
            .help("show staged contents' object name in the output")
            .short("s")
            .long("stage"))
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
    };
    match matches.subcommand_matches("hash-object") {
        Some(matches) => {
            let file = matches.value_of("file").unwrap();
            let w_opt = if let Some(_) = matches.args.get("write") { true } else { false };
            hash_object::hash_object(file, w_opt).unwrap();
        },
        None => {}
    };
    match matches.subcommand_matches("update-index") {
        Some(_) => {
            update_index::update_index().unwrap();
        },
        None => {}
    };
    match matches.subcommand_matches("ls-files") {
        Some(matches) => {
            let staged = if let Some(_) = matches.args.get("stage") { true } else { false };
            ls_files::ls_files(staged).unwrap();
        },
        None => {}
    }
}
