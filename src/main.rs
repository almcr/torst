mod torrent;
mod channels;
mod common;
mod peer;
mod protocol;
mod bitfield;
mod disk;
mod tracker;

use std::fs;

fn add_torrent(path: &str) {
  let content = std::fs::read_to_string(path).expect_err("can't open file");
}

fn main() {
  let torrent_file_path = "some/path.torrent";
}
