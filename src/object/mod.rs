
pub mod blob;
pub mod commit;
pub mod tree;

use crate::object::blob::Blob;
use crate::object::commit::Commit;
use crate::object::tree::{Tree, File};

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
        let obj_type = iter
            .next()
            .and_then(|o| String::from_utf8(o.to_vec()).ok())
            .and_then(|o| ObjectType::from(&o)).unwrap();
        iter
            .next()
            .and_then(|d| match obj_type {
                ObjectType::Blob => Blob::from_bytes(d).map(Object::Blob),
                // ObjectType::Commit => Commit::from_bytes(d).map(Object::Commit),
                // ObjectType::Tree => Tree::from_bytes(d).map(Object::Tree),
                ObjectType::Commit => Blob::from_bytes(d).map(Object::Blob),
                ObjectType::Tree => Blob::from_bytes(d).map(Object::Blob),
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
}

#[derive(Debug, Clone)]
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
