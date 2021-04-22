
use std::io;
use crate::refs;

pub fn checkout(branch: &str, new: bool) -> io::Result<()> {
    println!("checkout {} new? {}", branch, new);
    match new {
        true => {},
        false => {
            refs::switch_branch(branch)?;
        },
    };
    Ok(())
}
