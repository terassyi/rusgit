
use crate::object::blob::Blob;

#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub name: String,
    pub new: Blob,
    pub old: Blob,
}

impl DiffEntry {
    pub fn new(name: &str, new: Blob, old: Blob) -> DiffEntry {
        DiffEntry {
            name: String::from(name),
            new,
            old,
        }
    }

    pub fn is_modified(&self) -> bool {
        self.new.calc_hash() != self.old.calc_hash()
    }
}
