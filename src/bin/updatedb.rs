use std::collections::HashMap;
use std::os::unix::ffi::OsStrExt;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

type BufFile = BufWriter<File>;

fn index_file(
    path: &Path,
    parent_offset: u32,
    name_counter: &mut u32,
    name_db: &mut BufFile,
    meta_dict: &mut HashMap<u32, u32>,
) -> u32 {
    // Insert file into index
    let name_bytes = path.file_name().unwrap().as_bytes();
    name_db.write(name_bytes).unwrap();
    name_db.write(b"\0").unwrap();
    let my_offset = *name_counter;
    meta_dict.insert(my_offset, parent_offset);
    *name_counter += name_bytes.len() as u32 + 1;

    my_offset
}

fn index_dir(
    path: &Path,
    parent_offset: u32,
    name_counter: &mut u32,
    name_db: &mut BufFile,
    meta_dict: &mut HashMap<u32, u32>,
) {
    // Insert file into index
    if parent_offset == 0 {
        println!("Indexing at: {:?}", path);
    }
    let my_offset = index_file(path, parent_offset, name_counter, name_db, meta_dict);

    // Recurse
    let dir = std::fs::read_dir(path);
    match dir {
        Ok(dir) => {
            for ent in dir {
                let ent = ent.unwrap();
                if ent.file_type().unwrap().is_dir() {
                    index_dir(&ent.path(), my_offset, name_counter, name_db, meta_dict);
                } else {
                    index_file(&ent.path(), my_offset, name_counter, name_db, meta_dict);
                }
            }
        }
        Err(e) => {
            println!("Error while indexing {:?}: {:?}", path, e);
        }
    }

}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: updatedb <dir>");
    } else {
        let mut name_db =
            BufWriter::new(File::create("rlocate-names.db").expect("Failed to create name DB"));

        let mut name_counter = 0u32;
        let mut meta_dict = HashMap::new();
        index_dir(
            Path::new(&args[1]),
            0,
            &mut name_counter,
            &mut name_db,
            &mut meta_dict,
        );

        let mut meta_db = File::create("rlocate-meta.db").expect("Failed to create metadata DB");
        bincode::serialize_into(&mut meta_db, &meta_dict).expect("Failed to write metadata DB");
    }
}
