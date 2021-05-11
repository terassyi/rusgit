
use std::io;
use std::fs;
use std::str;
use std::path::Path;
use std::io::Write;
use std::io::Read;
use std::fs::File;
use crate::cmd::cat_file::hash_key_to_path;
use crate::object::commit::Commit;
use crate::object::tree::Tree;
use crate::cmd::GIT_BASE_DIR;
use crate::cmd::GIT_HEAD_FILE;
use crate::cmd::GIT_REFS_HEADS_DIR;
use crate::cmd::REFS_HEADS_DIR;

const REFS: &str = "ref:";

pub fn create_head() -> io::Result<()> {
    let mut file = File::create(GIT_HEAD_FILE)?; 
    let content = format!("{} {}/master", REFS, REFS_HEADS_DIR);
    file.write_all(&mut content.as_bytes())
}

pub fn read_head() -> io::Result<String> {
    let mut file = File::open(GIT_HEAD_FILE)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let content = String::from_utf8(buf).or(Err(io::Error::from(io::ErrorKind::NotFound)))?;
    let mut iter = content.split_whitespace();
    iter.next().ok_or(io::Error::from(io::ErrorKind::InvalidData))?;
    let refs = iter.next().ok_or(io::Error::from(io::ErrorKind::InvalidData))?;

    Ok(format!("{}/{}", GIT_BASE_DIR, refs))
}

fn update_head(name: &str) -> io::Result<String> {
    let mut file = File::create(GIT_HEAD_FILE)?; 
    let path = format!("{}/{}", REFS_HEADS_DIR, name);
    let content = format!("{} {}\n", REFS, path);
    file.write_all(&mut content.as_bytes())?;
    Ok(format!(".git/{}", path))
}

pub fn create_branch(name: &str) -> io::Result<()> {
    let ref_path = format!("{}/{}", GIT_REFS_HEADS_DIR, name);
    let head_path = read_head()?;
    let head_hash = read_ref(&head_path)?;
    let mut file = File::create(ref_path)?;
    file.write_all(head_hash.as_bytes())
}

pub fn switch_branch(name: &str) -> io::Result<()> {
    let path = update_head(name)?; // update .git/HEAD
    // update contents
    let head_hash = read_ref(&path)?;
    let commit = Commit::from_hash_file(&hash_key_to_path(&head_hash))?; 
    let tree = Tree::from_hash_file(&hash_key_to_path(&commit.tree))?; 
    tree.switch(".")?;
    // update .git/index
    Ok(())
}

pub fn read_head_branch() -> io::Result<String> {
    // get head branch name
    let mut file = File::open(GIT_HEAD_FILE)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let path = Path::new(str::from_utf8(&buf[0..buf.len()-1]).unwrap());
    let branch = path.file_name().ok_or(io::Error::from(io::ErrorKind::NotFound))?;

    Ok(String::from(branch.to_str().unwrap()))
}

pub fn show_branches() -> io::Result<Vec<String>> {
    let branchs = fs::read_dir(GIT_REFS_HEADS_DIR)?
                        .flat_map(|f| f)
                        .map(|f| {
                            let name = f.file_name();
                            String::from(name.to_str().unwrap())
                        })
                        .collect::<Vec<String>>();
    Ok(branchs)
}

pub fn read_ref(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let buf = if buf[buf.len() - 1] == b'\n' { buf[0..buf.len()-1].to_vec() } else { buf.to_vec() };
    let hash = String::from_utf8(buf).or(Err(io::Error::from(io::ErrorKind::InvalidInput)))?;
    Ok(hash)
}

fn write_ref(path: &str, hash: &str) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(hash.as_bytes())
}

pub fn update_ref(path: &str, hash: &str) -> io::Result<()> {
    write_ref(path, hash)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_read_head() {
        let refs = match super::read_head() {
            Ok(_) => true,
            Err(_) => false,
        };
        assert_eq!(refs, true);
    }
    #[test]
    fn test_read_head_branch() {
        let refs = match super::read_head_branch() {
            Ok(_) => true,
            Err(_) => false,
        };
        assert_eq!(refs, true);
    }
    #[test]
    fn test_show_branches() {
        let branches = super::show_branches().unwrap();
        for branch in branches.iter() { println!("{}", branch); }
        assert_eq!(branches.len() > 0, true);
    }
}
