
use std::io;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Read;
use crate::cmd::GITIGNORE;
use crate::cmd::GIT_BASE_DIR;
use crate::cmd::cat_file::{file_to_object, hash_key_to_path};

#[derive(Debug, Clone)]
pub struct GitIgnore {
    pub files: Vec<String>,
}

impl GitIgnore {
    pub fn new(files: Vec<String>) -> GitIgnore {
        GitIgnore {
            files
        }
    }

    pub fn read_gitignore() -> io::Result<GitIgnore> {
        let mut file = File::open(GITIGNORE)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let data = String::from_utf8(buf).or(Err(io::Error::from(io::ErrorKind::InvalidInput)))?;
        let mut lines = data.split('\n')
                        .filter(|&l| l != "")
                        .map(|l| {
                            if l.chars().next().unwrap() == '/' { format!(".{}", l) } else { format!("./{}", l) }
                            // String::from(l)
                        })
                        .collect::<Vec<String>>();
        if lines.len() != 0 && lines[lines.len()-1] == "" {
            lines.pop().ok_or(io::Error::from(io::ErrorKind::InvalidInput))?;
        }
        lines.push(format!("./{}", GIT_BASE_DIR));
        Ok(GitIgnore::new(lines))
    }

    pub fn is_ignored(&self, entry: &Path) -> bool {
        self.files.iter()
            .map(|f| entry.starts_with(f))
            .fold(false, |b, is_ignored| b || is_ignored )
    }

    pub fn walk_dir(&self) -> io::Result<Vec<String>> {
        let files = self.walk_dir_recursive(".", Vec::new())?
            .iter()
            .map(|f| f.replacen("./", "", 1))
            .collect::<Vec<String>>();
        Ok(files)
    }

    fn walk_dir_recursive(&self, path: &str, mut files: Vec<String>) -> io::Result<Vec<String>> {
        let list = fs::read_dir(path)?
                        .flat_map(|f| f)
                        .filter(|file| !self.is_ignored(file.path().as_path()));
        for l in list {
            let path_buf = l.path();
            let path = path_buf.as_path();
            let path_str = String::from(path.to_str().ok_or(io::Error::from(io::ErrorKind::InvalidInput))?);
            if path.is_dir() {
                files = self.walk_dir_recursive(&path_str, files.clone())?;
            } else {
                files.push(path_str);
            }
        }
        Ok(files)
    }
}



#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn test_read_gitignore() {
        let line = super::GitIgnore::read_gitignore().unwrap();
        assert_eq!(line.files[0], "./target");
        assert_eq!(line.files[line.files.len()-1], super::GIT_BASE_DIR);
    }
    #[test]
    fn test_is_ignored() {
        let gitignore = super::GitIgnore::read_gitignore().unwrap();
        let git = Path::new(".git");
        let target = Path::new("./target/hoge/fuga");
        assert_eq!(gitignore.is_ignored(git), true);
        assert_eq!(gitignore.is_ignored(target), true);
    }
    #[test]
    fn test_walk_dir_recursive() {
        let gitignore = super::GitIgnore::read_gitignore().unwrap();
        let files: Vec<String> = Vec::new();
        let f = gitignore.walk_dir_recursive(".", files).unwrap();
        assert_eq!(f[0], "./Cargo.toml");
    }
    #[test]
    fn test_walk_dir() {
        let gitignore = super::GitIgnore::read_gitignore().unwrap();
        let files = gitignore.walk_dir().unwrap();
        assert_eq!(files[0], "Cargo.toml");

    }
    #[test]
    fn test_start_with() {
        let target_path = Path::new("./target/hoge/fuga");
        let target = Path::new("./target");
        assert_eq!(target_path.starts_with(target), true);
    }

}
