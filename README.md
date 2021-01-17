# rlocate

File locator daemon written in Rust

All of this is very WIP, Database path is hardcoded to `./rlocate-names.db` and `./rlocate-meta.db`

Very little effort was spent on optimization

## Build instructions & Usage

```sh
# Clone
git clone git@github.com:Wazzaps/rlocate.git

# Build (I assume Rust is installed)
cargo build --release

# Create DB and spawn daemon
mkdir index_dir && cd index_dir
../target/release/updatedb /usr
../target/release/located &

# Usage
../target/release/locate Head
../target/release/locate gnome
../target/release/locate libc.so
```

## (Very rough) Benchmarks

|                               | rlocate                       | GNU mlocate                                    |
| ----------------------------- | ----------------------------- | ---------------------------------------------- |
| index of my `/usr`            | `./updatedb /usr` - 4.3 sec   | `updatedb -U /usr -o mlocate.db` - 3 sec       |
| search of `Head`              | `./locate Head` - 10ms        | `locate -d ./mlocate.db Head` - 248 ms         |
| search of `Head` (10 results) | `./locate Head \| head` - 2ms | `locate -d ./mlocate.db Head \| head` - 195 ms |
| background RAM usage          | 9.4 MiB                       | 0 MiB                                          |
| db size                       | 14.35 MiB                     | 13.47 MiB                                      |

## Critical Missing Features

- Don't go into `/sys`, `/proc`, etc. when indexing
  - Don't travel into mount points
  - Specify multiple roots (for indexing external drives)
- Expose case insensitivity as flag
- Don't stall on greedy regex (i.e. `.*`), cap matches to `PATH_MAX`
- Specify DB path

## Nice to have features

- Mmap the metadata db too
- Send messages / panics back to client
- Describe regex support in `--help`