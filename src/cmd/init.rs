use std::io;
use std::fs;
use std::fs::File;

const RUSGIT_BASE_DIR: &str = ".rusgit";
const RUSGIT_OBJECTS_DIR: &str = ".rusgit/objects";
const RUSGIT_REFS_DIR: &str = ".rusgit/refs";
const RUSGIT_HEAD_FILE: &str = ".rusgit/HEAD";

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
