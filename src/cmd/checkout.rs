
use std::io;
use crate::refs;

pub fn checkout(branch: &str, new: bool) -> io::Result<()> {
    match new {
        true => {
            refs::create_branch(branch)?;
            refs::switch_branch(branch)?;
        },
        false => {
            refs::switch_branch(branch)?;
        },
    };
    Ok(())
}
