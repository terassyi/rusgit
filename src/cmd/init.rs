use std::io;
use std::fs;
use std::fs::File;

use crate::refs::create_head;
use crate::cmd::GIT_BASE_DIR;
use crate::cmd::GIT_OBJECTS_DIR;
use crate::cmd::GIT_REFS_DIR;
use crate::cmd::GIT_HEAD_FILE;

pub fn init_rusgit() -> io::Result<()> {
    // mkdir .rugit dir
    if let Err(_) = fs::create_dir(GIT_BASE_DIR) {
        println!("Already initialized for rusgit repository.");
        return Ok(())
    }
    fs::create_dir(GIT_OBJECTS_DIR)?;
    fs::create_dir(GIT_REFS_DIR)?;
    create_head()?;
    Ok(())
}
