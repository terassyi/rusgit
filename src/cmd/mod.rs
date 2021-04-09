pub mod init;
pub mod cat_file;
pub mod hash_object;
pub mod update_index;
pub mod ls_files;

pub const RUSGIT_BASE_DIR: &str = ".rusgit";
pub const RUSGIT_OBJECTS_DIR: &str = ".rusgit/objects";
pub const RUSGIT_INDEX: &str = ".rusgit/index";
const RUSGIT_REFS_DIR: &str = ".rusgit/refs";
const RUSGIT_HEAD_FILE: &str = ".rusgit/HEAD";
