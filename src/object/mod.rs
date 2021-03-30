
pub mod blob;
pub mod commit;
pub mod tree;

use std::str;
use std::io;
use std::fs;
use std::io::Write;
use libflate::zlib::Encoder;

use crate::object::blob::Blob;
use crate::object::commit::Commit;
use crate::object::tree::{Tree, File};
use crate::cmd::RUSGIT_OBJECTS_DIR;

const BLOB: &str = "blob";
const COMMIT: &str = "commit";
const TREE: &str = "tree";

#[derive(Debug, Clone)]
pub enum Object {
    Blob(Blob),
    Commit(Commit),
    Tree(Tree),
}

impl Object {
    pub fn new(data: &[u8]) -> Self {
        let mut iter = data.splitn(2, |&b| b == b'\0');
        let obj_type_str = str::from_utf8(iter
            .next().unwrap()).unwrap();
        let obj_type = obj_type_str.split_whitespace()
            .next()
            .and_then(|o| ObjectType::from(&o)).unwrap();
        iter
            .next()
            .and_then(|d| match obj_type {
                ObjectType::Blob => Blob::from(d).map(Object::Blob),
                ObjectType::Commit => Commit::from(d).map(Object::Commit),
                ObjectType::Tree => Tree::from(d).map(Object::Tree),
            }
        ).unwrap()
    }

    pub fn typ(&self) -> ObjectType {
        match self {
            Object::Blob(blob) => blob.typ(),
            Object::Commit(commit) => commit.typ(),
            Object::Tree(tree) => tree.typ(),
        }
    }

    pub fn size(data: &[u8]) -> Option<usize> {
        let mut iter = data.splitn(2, |&b| b == b'\0');
        let mut size_iter = str::from_utf8(iter.next()?).ok()?
                    .split_whitespace();
        size_iter.next()?;
        let size = size_iter.next()?
                    .parse::<usize>().ok()?;
        Some(size)
    }

    pub fn write(&self) -> io::Result<()> {
        let hash = hex::encode(self.calc_hash());
        let (sub_dir, name) = hash.split_at(2);
        let dir = format!("{}/{}", RUSGIT_OBJECTS_DIR, sub_dir);
        if !fs::metadata(&dir).is_ok() {
            fs::create_dir(&dir)?;
        }
        let file_path = format!("{}/{}", dir, name);
        let mut file = fs::File::create(file_path)?;
        let mut encoder = Encoder::new(Vec::new())?;
        encoder.write_all(&self.as_bytes())?;
        let data = encoder.finish().into_result()?;
        file.write(&data)?;
        Ok(())
    }

    pub fn calc_hash(&self) -> Vec<u8> {
        match self {
            Object::Blob(blob) => blob.calc_hash(),
            Object::Commit(commit) => commit.calc_hash(),
            Object::Tree(tree) => tree.calc_hash(),
        } 
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Object::Blob(blob) => blob.as_bytes(),
            Object::Commit(commit) => commit.as_bytes(),
            Object::Tree(tree) => tree.as_bytes(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ObjectType {
    Blob,
    Commit,
    Tree,
}

impl ObjectType {
    fn from(data: &str) -> Option<Self> {
        let mut hdr = data.split_whitespace();
        match hdr.next()? {
            BLOB => Some(ObjectType::Blob),
            COMMIT => Some(ObjectType::Commit),
            TREE => Some(ObjectType::Tree),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            ObjectType::Blob => String::from(BLOB),
            ObjectType::Commit => String::from(COMMIT),
            ObjectType::Tree => String::from(TREE),
        }
    }
}
