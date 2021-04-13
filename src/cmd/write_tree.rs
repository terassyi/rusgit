use std::io;
use crate::object::tree;
use crate::object::Object;

pub fn write_tree() -> io::Result<()> {
    let tree = tree::write_tree()?;
    let obj = Object::Tree(tree);
    obj.write()
}
