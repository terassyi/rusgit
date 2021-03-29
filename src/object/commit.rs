use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use std::str;

use crate::object::{Object, ObjectType};

#[derive(Debug, Clone)]
pub struct Commit {
    pub tree: String,
    pub parent: Option<String>,
    pub author: User,
    pub commiter: User,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub email: String,
    pub timestamp: DateTime<FixedOffset>,
}

impl User {
    fn new(name: &str, email: &str, time: DateTime<FixedOffset>) -> Self {
        User {
            name: String::from(name),
            email: String::from(email),
            timestamp: time,
        }
    }

    pub fn from(data: &str) -> Option<Self> {
        let mut iter = data.split_whitespace();
        // author or commiter
        iter.next().unwrap();
        // name
        let name = String::from(iter.next().unwrap());
        let email = iter.next().map(|d| String::from(d.trim_matches(|d| d == '<' || d == '>')))?;
        let ts = Utc.timestamp(iter.next().and_then(|x| x.parse::<i64>().ok())?, 0);
        let offset = iter.next().and_then(|d| d.parse::<i32>().ok())
            .map(|x| {
                if x < 0 {
                    FixedOffset::west(x / 100 * 60 * 60)
                } else {
                    FixedOffset::east(x / 100 * 60 * 60)
                }
            })?;
        Some(User {
            name,
            email,
            timestamp: offset.from_utc_datetime(&ts.naive_utc())
        })
    }
}

impl Commit {
    pub fn new(tree: &str, parent: Option<&str>, author: User, commiter: User, message: &str) -> Self {
        match parent {
            Some(parent) => {
                Commit {
                    tree: String::from(tree),
                    parent: Some(String::from(parent)),
                    author,
                    commiter,
                    message: String::from(message),
                }
            },
            None => {
                Commit {
                    tree: String::from(tree),
                    parent: None,
                    author,
                    commiter,
                    message: String::from(message),
                }

            }
        }
    }

    pub fn from(data: &[u8]) -> Option<Self> {
        let mut lines = data.split(|&d| d == b'\n').filter(|&d| d != b"");
        let c = lines.clone().count();
        let tree = str::from_utf8(lines.next()?)
            .ok()?
            .split_whitespace()
            .last()?;
        let parent = if c == 6 {
            // parent exists
            Some(String::from(str::from_utf8(lines.next()?)
            .ok()?
            .split_whitespace()
            .last()?))
        } else {
            None
        };
        let author = User::from(str::from_utf8(lines.next()?).ok()?)?;
        let commiter = User::from(str::from_utf8(lines.next()?).ok()?)?;
        lines.next()?;
        let message = String::from_utf8(lines.next()?.to_vec()).ok()?;
        Some(Commit {
            tree: String::from(tree),
            parent,
            author,
            commiter,
            message,
        })
    }

    pub fn typ(&self) -> ObjectType {
        ObjectType::Commit
    }
}

#[cfg(test)]
mod tests {
    use super::User;
    #[test]
    fn test_user_from() {
        let usr_str = "author terassyi <iscale821@gmail.com> 1616834749 +0900";
        let user = User::from(usr_str).unwrap();
        assert_eq!(user.name, String::from("terassyi"));
        assert_eq!(user.email, String::from("iscale821@gmail.com"));
    }
    use super::Commit;
    #[test]
    fn test_commit_from() {
        let commit_str = "tree bd41dfafd2299ddc08ff789c8a777ff0b8ce9e4c\nparent a213f26901a29e8fecf60da136c31d61dd41544b\nauthor terassyi <iscale821@gmail.com> 1616834749 +0900\ncommitter terassyi <iscale821@gmail.com> 1616834749 +0900\n\r\nadd init cmd\n";
        let commit = Commit::from(commit_str.as_bytes()).unwrap();
        assert_eq!(commit.tree, String::from("bd41dfafd2299ddc08ff789c8a777ff0b8ce9e4c"));
        assert_eq!(commit.parent, Some(String::from("a213f26901a29e8fecf60da136c31d61dd41544b")));
        assert_eq!(commit.message, String::from("add init cmd"));
    }
}
