use std::io;
use std::fs;
use std::fs::File;

use crate::cmd::RUSGIT_BASE_DIR;
use crate::cmd::RUSGIT_OBJECTS_DIR;
use crate::cmd::RUSGIT_REFS_DIR;
use crate::cmd::RUSGIT_HEAD_FILE;

pub fn init_rusgit() -> io::Result<()> {
    // mkdir .rugit dir
    if let Err(_) = fs::create_dir(RUSGIT_BASE_DIR) {
        println!("Already initialized for rusgit repository.");
        return Ok(())
    }
    fs::create_dir(RUSGIT_OBJECTS_DIR)?;
    fs::create_dir(RUSGIT_REFS_DIR)?;
    File::create(RUSGIT_HEAD_FILE)?;
    Ok(())
}
