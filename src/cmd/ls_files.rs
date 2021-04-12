use std::io;
use std::fs::File;
use std::io::Read;
use crate::index::read_index;

pub fn ls_files(staged: bool) -> io::Result<()> {
    let index_path = ".git/index";
    let index = read_index(index_path)?;
    if staged {
        print!("{}", index);
    } else {
        for e in index.entries {
            println!("{}", e.name);
        }
    }
    Ok(())
}
