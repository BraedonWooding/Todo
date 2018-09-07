use std::path::{PathBuf};
use std::env::{home_dir};
use std::io::{Write, stdout};

use errors::*;

pub fn get_file_path() -> Result<PathBuf> {
    match home_dir() {
        Some(mut path) => {
            path.push("_todo_lists/");
            Ok(path)
        },
        None => {
            bail!("Invalid Path")
        },
    }
}