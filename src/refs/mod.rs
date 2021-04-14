
use std::io;
use std::str;
use std::path::Path;
use std::io::Write;
use std::io::Read;
use std::fs::File;
use crate::cmd::GIT_BASE_DIR;
use crate::cmd::GIT_HEAD_FILE;
use crate::cmd::GIT_REFS_HEADS_DIR;

const REFS: &str = "ref:";

pub fn create_head() -> io::Result<()> {
    let mut file = File::create(GIT_HEAD_FILE)?; 
    let content = format!("{} {}/master", REFS, GIT_REFS_HEADS_DIR);
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
    println!("read_head {}", refs);

    Ok(format!("{}/{}", GIT_BASE_DIR, refs))
}

pub fn read_head_branch() -> io::Result<String> {
    let mut file = File::open(GIT_HEAD_FILE)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let path = Path::new(str::from_utf8(&buf).unwrap());
    let branch = path.file_name().ok_or(io::Error::from(io::ErrorKind::NotFound))?;
    Ok(String::from(branch.to_str().unwrap()))
}

pub fn read_ref(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
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
}
