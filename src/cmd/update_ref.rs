
use std::io;
use crate::refs;

pub fn update_ref(path: &str, hash: &str) -> io::Result<()> {
    refs::update_ref(path, hash)
}
