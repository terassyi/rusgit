
use std::io;
use crate::index;
use crate::cmd::GIT_INDEX;

pub fn status() -> io::Result<()> {
    let index = index::read_index(GIT_INDEX)?;
    let diff_entries = index.diff().unwrap();
    if diff_entries.len() == 0 {
        println!("nothing to commit, working tree clean");
        return Ok(());
    }
    println!("Changes not staged for commit:");
    for d in diff_entries.iter() {
        println!("\tmodified:\t{}", d.name);
    }

    let untracked = index.untracked_files()?;
    if untracked.len() != 0 {
        println!("Untracket fules:");
        for f in untracked {
            println!("\t{}", f);
        }
    }

    Ok(())
}
