
use std::io;

pub fn diff() -> io::Result<()> {
    /*
        1. read index and get entries
        2. get blob objects of each entries
        3. decode contents of blob
        4. open the file
        5. compare
     */
    println!("diff");
    Ok(())
}
