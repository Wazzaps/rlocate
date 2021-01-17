use std::{io::{BufRead, BufReader, Read, Write}, os::unix::net::UnixStream, process::exit};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 || args[1] == "--help" {
        eprintln!("Usage: locate <query>");
        exit(1);
    } else {
        let stream = UnixStream::connect("/tmp/everything.sock");
        match stream {
            Ok(mut stream) => {
                stream.write_all(args[1].as_bytes()).expect("Failed to send query");
                stream.write_all(b"\n").expect("Failed to send query");
                let reader = BufReader::new(&stream);

                let stdout = std::io::stdout();
                let mut stdout = stdout.lock();
                for line in reader.lines() {
                    if let Err(_) = stdout.write_all(line.expect("Connection with daemon broken").as_bytes()) {
                        break;
                    }
                    if let Err(_) = stdout.write_all(b"\n") {
                        break;
                    }
                }
            }
            Err(_) => {
                eprintln!("Failed to connect to daemon, is `located` running?");
                exit(1);
            }
        }
    }
}