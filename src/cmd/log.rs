
use std::io;
use chrono::Weekday;
use chrono::Datelike;
use crate::refs;
use crate::cmd::cat_file;
use crate::object::Object;
use crate::object::commit::Commit;

pub fn log() -> io::Result<()> {
    let commit = refs::read_head()
                    .and_then(|ref_path| refs::read_ref(&ref_path))
                    .and_then(|hash| Ok(cat_file::hash_key_to_path(&hash)))
                    .and_then(|path| cat_file::file_to_object(&path))
                    .and_then(|obj| { 
                        match obj {
                            Object::Commit(commit) => Ok(commit),
                            _ => Err(io::Error::from(io::ErrorKind::InvalidInput))
                        }
                    })?;
    let output = log_recursive(&commit, Vec::new()).ok_or(io::Error::from(io::ErrorKind::InvalidInput))?;
    for o in output {
        print!("{}", o);
    }

    Ok(())
}

fn log_recursive(commit: &Commit, mut output: Vec<String>) -> Option<Vec<String>> {
    output.push(format_log(commit).ok()?);
    match &commit.parent {
        Some(parent) => {
            let path = cat_file::hash_key_to_path(&parent);
            let parent_commit = cat_file::file_to_object(&path)
                    .and_then(|obj| { 
                        match obj {
                            Object::Commit(commit) => Ok(commit),
                            _ => Err(io::Error::from(io::ErrorKind::InvalidInput))
                        }
                    }).ok()?;
            log_recursive(&parent_commit, output)
        },
        None => Some(output)
    }
}

fn format_log(commit: &Commit) -> io::Result<String> {
    let weekday = commit.author.timestamp.weekday();
    let date = commit.author.timestamp.format("%d %H:%M:%S %Y %Z").to_string();
    let output = format!("commit {}\nAuthor: {} <{}>\nDate:\t{} {} {}\n\n\t{}\n", 
        hex::encode(commit.calc_hash()), 
        commit.author.name, 
        commit.author.email,
        format_weekday(&weekday),
        format_month(commit.author.timestamp.month()),
        date,
        commit.message
    );
    Ok(output)
}

fn format_weekday(weekday: &Weekday) -> &str {
    match weekday {
        Weekday::Mon => "Mon",
        Weekday::Tue => "Tue",
        Weekday::Wed => "Wed",
        Weekday::Thu => "Thu",
        Weekday::Fri => "Fri",
        Weekday::Sat => "Sat",
        Weekday::Sun => "Sun",
    }
}

fn format_month(month: u32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    }
}
