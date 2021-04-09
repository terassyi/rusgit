use std::io;
use std::fs::File;
use std::io::Read;
use crate::index::{Entry, Index};

pub fn ls_files(staged: bool) -> io::Result<()> {
    let index_path = ".git/index";
    let mut file = File::open(index_path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let index = Index::from(&buf).ok_or(io::Error::from(io::ErrorKind::InvalidInput))?;
    if staged {
        println!("{}", index);
    } else {
        for e in index.entries {
            println!("{}", e.name);
        }
    }
    Ok(())
}
