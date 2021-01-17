use std::{io::BufRead, os::unix::prelude::OsStrExt};
use std::{collections::HashMap, fs::File};
use std::{io::BufReader, os::unix::net::UnixListener};
use std::{
    io::{BufWriter, Write},
    os::unix::net::UnixStream,
    time::Instant,
};

use memmap::MmapOptions;
use regex::bytes::{Match, Regex};

#[path = "../sock_path.rs"]
mod sock_path;

fn find_null_fwd(buf: &[u8], offset: usize) -> Option<usize> {
    for (i, ch) in buf.iter().enumerate().skip(offset) {
        if *ch == 0 {
            return Some(i);
        }
    }
    None
}

fn find_null_rev(buf: &[u8], offset: usize) -> Option<usize> {
    for i in (0..offset).rev() {
        if buf[i] == 0 {
            return Some(i + 1);
        }
    }
    Some(0)
}

struct Entry {
    name: Vec<u8>,
    parent_off: u32,
}

fn match_to_entry(m: Match, names_db: &[u8], meta_db: &HashMap<u32, u32>) -> Entry {
    let filename_start = find_null_rev(names_db, m.start()).unwrap();
    let filename_end = find_null_fwd(names_db, m.end()).unwrap();

    let mut bolded_name = vec![];
    bolded_name.extend_from_slice(&names_db[filename_start..m.start()]);
    // bolded_name.extend_from_slice(b"<b>");
    bolded_name.extend_from_slice(&names_db[m.start()..m.end()]);
    // bolded_name.extend_from_slice(b"</b>");
    bolded_name.extend_from_slice(&names_db[m.end()..filename_end]);

    Entry {
        name: bolded_name,
        parent_off: meta_db[&(filename_start as u32)],
    }
}

fn offset_to_entry(offset: usize, names_db: &[u8], meta_db: &HashMap<u32, u32>) -> Entry {
    let filename_end = find_null_fwd(names_db, offset).unwrap();

    Entry {
        name: names_db[offset..filename_end].to_vec(),
        parent_off: meta_db[&(offset as u32)],
    }
}

fn recurse_entry(
    ent: &Entry,
    names_db: &[u8],
    meta_db: &HashMap<u32, u32>,
    target: &mut Vec<Vec<u8>>,
) {
    target.push(ent.name.clone());
    if ent.parent_off != 0 {
        let parent = offset_to_entry(ent.parent_off as usize, names_db, meta_db);
        recurse_entry(&parent, names_db, meta_db, target);
    }
}

fn locate<W: Write>(query: &str, output: &mut W, names_db: &[u8], meta_db: &HashMap<u32, u32>) {
    let re = Regex::new(query).unwrap();
    let entries = re
        .find_iter(names_db)
        .map(|m| match_to_entry(m, names_db, meta_db));
    for ent in entries {
        let mut full_path = vec![];
        recurse_entry(&ent, names_db, meta_db, &mut full_path);
        let full_path: Vec<&[u8]> = full_path.iter().rev().map(|v| v.as_slice()).collect();
        let full_path = full_path.join(&b'/');
        let full_path_str = String::from_utf8_lossy(full_path.as_slice());
        if writeln!(output, "/{}", full_path_str).is_err() {
            break;
        }
    }
}

fn client_handler(stream: UnixStream, names_db: &[u8], meta_db: &HashMap<u32, u32>) {
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    let mut query = String::new();
    reader.read_line(&mut query).unwrap();
    query.pop(); // Remove \n

    let start_time = Instant::now();
    locate(&query, &mut writer, names_db, meta_db);

    println!(
        "Query {:?} took {}ms",
        &query,
        Instant::now().duration_since(start_time).as_millis()
    );
}

fn main() {
    println!("Loading names...");
    let name_db = File::open("rlocate-names.db").expect("Failed to open names DB");
    let name_mmap = unsafe {
        MmapOptions::new()
            .map(&name_db)
            .expect("Failed to map names DB")
    };

    println!("Loading metadata...");
    let mut meta_db = File::open("rlocate-meta.db").expect("Failed to open metadata DB");
    let meta_dict: HashMap<u32, u32> =
        bincode::deserialize_from(&mut meta_db).expect("Failed to read metadata DB");

    println!("Ready!");

    let sock_path = sock_path::get();

    if sock_path.exists() {
        std::fs::remove_file(&sock_path).expect("Failed to remove existing socket file");
    }

    let listener = UnixListener::bind(&sock_path).expect("Failed to bind socket");
    println!("Listening on {}", String::from_utf8_lossy(sock_path.as_os_str().as_bytes()));
    crossbeam::scope(|scope| {
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                scope.spawn(|_| client_handler(stream, &name_mmap, &meta_dict));
            }
        }
    })
    .unwrap();
}
