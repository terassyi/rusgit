
use std::io;
use crate::index;
use crate::cmd::GIT_INDEX;

pub fn diff() -> io::Result<()> {
    /*
        1. read index and get entries
        2. get blob objects of each entries
        3. decode contents of blob
        4. open the file
        5. compare
     */
    println!("diff");
    let index = index::read_index(GIT_INDEX)?;
    let diff_entries = index.diff()?;
    for entry in diff_entries {
        println!("diff --git a/{} b/{}", entry.name, entry.name);
        if entry.is_mode_modified() {
            println!("old mode {}", entry.old_mode);
            println!("new mode {}", entry.new_mode);
        }
        // println!("index {}..{} {}", &hex::encode(entry.old.calc_hash())[0..7], &hex::encode(entry.new.calc_hash())[0..7], entry.new_mode);
        let new: Vec<&str> = entry.new.content.split('\n').collect();
        let old: Vec<&str> = entry.old.content.split('\n').collect();
        if entry.is_contents_modified() {
            if entry.is_mode_modified() {
                println!("index {}..{}", &hex::encode(entry.old.calc_hash())[0..7], &hex::encode(entry.new.calc_hash())[0..7]);
            } else {
                println!("index {}..{} {}", &hex::encode(entry.old.calc_hash())[0..7], &hex::encode(entry.new.calc_hash())[0..7], entry.new_mode);
            }
            println!("--- a/{}", entry.name);
            println!("+++ b/{}", entry.name);
            let result = entry.compare();
            for r in result {
                match r {
                    wu_diff::DiffResult::Common(elm) => {},
                    wu_diff::DiffResult::Removed(elm) => {
                        match elm.old_index {
                            Some(i) => println!("- {}", old[i]),
                            None => {},
                        }
                    },
                    wu_diff::DiffResult::Added(elm) => {
                        match elm.new_index {
                            Some(i) => println!("+ {}", new[i]),
                            None => {},
                        }
                    },
                }
            }
        }
    }
    Ok(())
}
