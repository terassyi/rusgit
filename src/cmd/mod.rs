pub mod init;
pub mod cat_file;
pub mod hash_object;
pub mod update_index;
pub mod ls_files;
pub mod add;
pub mod write_tree;
pub mod commit_tree;
pub mod update_ref;
pub mod commit;
pub mod log;

pub const RUSGIT_BASE_DIR: &str = ".rusgit";
pub const RUSGIT_OBJECTS_DIR: &str = ".rusgit/objects";
pub const RUSGIT_INDEX: &str = ".rusgit/index";
const RUSGIT_REFS_DIR: &str = ".rusgit/refs";
pub const RUSGIT_REFS_HEADS_DIR: &str = ".rusgit/refs/heads";
pub const RUSGIT_REFS_REMOTES_DIR: &str = ".rusgit/refs/remotes";
const RUSGIT_HEAD_FILE: &str = ".rusgit/HEAD";

pub const GIT_BASE_DIR: &str = ".git";
pub const GIT_OBJECTS_DIR: &str = ".git/objects";
pub const GIT_INDEX: &str = ".git/index";
pub const GIT_REFS_DIR: &str = ".git/refs";
pub const GIT_REFS_HEADS_DIR: &str = ".git/refs/heads";
pub const GIT_REFS_REMOTES_DIR: &str = ".git/refs/remotes";
pub const GIT_HEAD_FILE: &str = ".git/HEAD";
