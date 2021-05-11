
use std::io;
use crate::refs;

pub fn branch(branch_name: Option<&str>) -> io::Result<()> {
    match branch_name {
        Some(branch_name) => {
            refs::create_branch(branch_name)?;
        },
        None => {
            // show branch
            let branches = refs::show_branches()?;
            let current = refs::read_head_branch()?;
            for b in branches.iter() {
                if b == &current {
                    println!("* {}", b);
                } else {
                    println!("  {}", b);
                }
            }
        },
    };
    Ok(())
}
