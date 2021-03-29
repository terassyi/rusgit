
use crate::object::{Object, ObjectType};

#[derive(Debug, Clone)]
pub struct File {
    pub mode: usize,
    pub name: String,
    pub hash: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Tree {

}

impl Tree {

    pub fn typ(&self) -> ObjectType {
        ObjectType::Tree
    }
}
