
use std::io;
use std::path::Path;
use crate::object::tree;
use crate::object::commit;
use crate::object::Object;
use crate::refs;

pub fn commit(message: &str) -> io::Result<()> {
    /* console output
        [master ca77114] second git
        1 file changed, 1 insertion(+)
        create mode 100644 .dockerignore
    */
    // git write-tree
    let tree = tree::write_tree()?;
    println!("success write tree");
    let obj = Object::Tree(tree);
    let hash = obj.write()?;
    println!("success write object");

    // git commit-tree
    // look up parent commit
    let ref_path = refs::read_head()?;
    println!("success read object");
    let parent_res = refs::read_ref(&ref_path);
    let parent: Option<&str> = match  parent_res {
        Ok(ref p) => Some(p),
        Err(e) => {
            if e.kind() != io::ErrorKind::NotFound {
                return Err(e);
            }
            None
        }
    };

    let name = "terassyi";
    let email = "example@terassyi.net";
    let commit = commit::commit_tree(name, email, &hash, message, parent)?;
    let obj = Object::Commit(commit);
    let commit_hash = obj.write()?;

    // git update-ref
    refs::update_ref(&ref_path, &commit_hash)?;

    // output
    let branch = Path::new(&ref_path).file_name().unwrap().to_str().unwrap();
    println!("[{} {}] {}", branch, &commit_hash[0..7], message);
    Ok(())
}
