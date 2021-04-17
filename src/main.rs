extern crate clap;
// extern crate walkdir;

use clap::{App, Arg, SubCommand};

use crate::cmd::init;
use crate::cmd::cat_file;
use crate::cmd::hash_object;
use crate::cmd::update_index;
use crate::cmd::ls_files;
use crate::cmd::add;
use crate::cmd::write_tree;
use crate::cmd::commit_tree;
use crate::cmd::update_ref;
use crate::cmd::commit;
use crate::cmd::log;
use crate::cmd::status;
use crate::cmd::diff;
use crate::cmd::branch;

pub mod cmd;
mod object;
mod index;
mod refs;

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
                .help("do not ignore new files")
                .long("add")
                .takes_value(true)
                // .empty_values(true)
                .default_value("")
                .required(true))
            .arg(Arg::with_name("cacheinfo")
                .help("add the specified entry to the index")
                .long("cacheinfo")
                .takes_value(true)
                .multiple(true))
        )
        .subcommand(SubCommand::with_name("ls-files")
            .about("list up files")
            .arg(Arg::with_name("stage")
            .help("show staged contents' object name in the output")
            .short("s")
            .long("stage"))
        )
        .subcommand(SubCommand::with_name("add")
            .about("stage files")
            .arg(Arg::with_name("file")
            .help("stage files")
            .multiple(true)
            .required(true))
        )
        .subcommand(SubCommand::with_name("write-tree")
            .about("write index as tree object")
        )
        .subcommand(SubCommand::with_name("commit-tree")
            .about("commit tree object")
            .arg(Arg::with_name("sha1")
            .help("sha1 value")
            .required(true))
            .arg(Arg::with_name("parent")
            .help("parent commit object")
            .takes_value(true)
            .short("p"))
            .arg(Arg::with_name("message")
            .help("commit message")
            .takes_value(true)
            .short("m"))
        )
        .subcommand(SubCommand::with_name("update-ref")
            .about("update reference")
            .arg(Arg::with_name("ref")
            .help("reference path")
            .required(true))
            .arg(Arg::with_name("sha1")
            .help("sha1 value")
            .required(true))
        )
        .subcommand(SubCommand::with_name("commit")
            .about("commit")
            .arg(Arg::with_name("message")
            .help("commit message")
            .short("m")
            .takes_value(true)
            .required(true)) // message must be specified
        )
        .subcommand(SubCommand::with_name("log")
            .about("log")
        )
        .subcommand(SubCommand::with_name("status")
            .about("show status")
        )
        .subcommand(SubCommand::with_name("diff")
            .about("show diff")
        )
        .subcommand(SubCommand::with_name("branch")
            .about("show branch")
            .arg(Arg::with_name("branch-name")
            .help("branch name")
            .takes_value(true))
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
        Some(matches) => {
            match matches.values_of("cacheinfo") {
                Some(val) => {
                    let values: Vec<&str> = val.collect();
                    if values.len() != 3 {
                        println!("you must specified <mode> <object> <path>");
                    }
                    update_index::update_index(values[2], Some(values[0]), Some(values[1])).unwrap();
                },
                None => {},
            }
            match matches.value_of("add") {
                Some(path) => {
                    if path == "" {
                        println!("add value is not set");
                        return;
                    }
                    update_index::update_index(path, None, None).unwrap();
                },
                None => {}
            };
        },
        None => {}
    };
    match matches.subcommand_matches("ls-files") {
        Some(matches) => {
            let staged = if let Some(_) = matches.args.get("stage") { true } else { false };
            ls_files::ls_files(staged).unwrap();
        },
        None => {}
    };
    match matches.subcommand_matches("add") {
        Some(matches) => {
            let files: Vec<&str> = matches.values_of("file").unwrap().collect();
            add::add(files).unwrap();
        },
        None => {}
    };
    match matches.subcommand_matches("write-tree") {
        Some(_) => write_tree::write_tree().unwrap(),
        None => {}
    };
    match matches.subcommand_matches("commit-tree") {
        Some(matches) => {
            let sha1 = matches.value_of("sha1").unwrap();
            let commit = commit_tree::commit_tree(sha1, matches.value_of("parent"), matches.value_of("message")).unwrap();
            println!("{}", commit);
        },
        None => {},
    };
    match matches.subcommand_matches("update-ref") {
        Some(matches) => {
            let path = matches.value_of("ref").unwrap();
            let hash = matches.value_of("sha1").unwrap();
            update_ref::update_ref(path, hash).unwrap();
        },
        None => {},
    };
    match matches.subcommand_matches("commit") {
        Some(matches) => {
            let message = matches.value_of("message").unwrap();
            commit::commit(message).unwrap();
        },
        None => {},
    };
    match matches.subcommand_matches("log") {
        Some(_) => log::log().unwrap(),
        None => {},
    };
    match matches.subcommand_matches("status") {
        Some(_) => status::status().unwrap(),
        None => {},
    };
    match matches.subcommand_matches("diff") {
        Some(_) => diff::diff().unwrap(),
        None => {},
    };
    match matches.subcommand_matches("branch") {
        Some(matches) => {
            let branch_name = matches.value_of("branch-name");
            branch::branch(branch_name).unwrap()
        },
        None => {},
    };
}
