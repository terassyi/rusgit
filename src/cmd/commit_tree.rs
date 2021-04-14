use std::io;
use crate::object::commit;
use crate::object::Object;

pub fn commit_tree(sha1: &str, parent: Option<&str>, message: Option<&str>) -> io::Result<String> {
    // message is option, but for commiting, message must be specified.
    let message = message.ok_or(io::Error::from(io::ErrorKind::NotFound))?;
    // TODO read config
    let name = "terassyi";
    let email = "example@terassyi.net";
    let commit = commit::commit_tree(name, email, sha1, message, parent)?;
    let obj = Object::Commit(commit);
    obj.write()
}
