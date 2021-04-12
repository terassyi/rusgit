use std::io;
use crate::cmd::hash_object;
use crate::cmd::update_index;

pub fn add(files: Vec<&str>) -> io::Result<()> {
    // . is not support to stage all files
    for file in files.iter() {
        hash_object::hash_object(file, true)?;
        update_index::update_index(file, None, None)?;
    }
    Ok(())
}
