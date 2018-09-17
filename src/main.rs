#![recursion_limit = "1024"] // for error_chain
#![allow(dead_code)]

extern crate clap;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate dialoguer;
#[macro_use]
extern crate indoc;
extern crate ctrlc;
extern crate glob;
#[macro_use]
extern crate error_chain;
extern crate termion;


use std::fs::{DirBuilder, canonicalize};
use std::path::{PathBuf};
use std::io::Write;

mod todo_list;
mod cli;
mod select_helper;
mod init_communism;
mod window;
mod util;
pub use util::*;
use window::*;

use errors::*;
mod errors {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            Toml(::toml::ser::Error);
            Glob(::glob::GlobError);
            Fmt(::std::fmt::Error);
        }
    }
}

// Note: can't use macro quick_main!(run)
// since we want to force to show cursor on exit.
fn main() {
    if let Err(ref e) = run() {
        use error_chain::ChainedError;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        WindowView::new().expect(errmsg).set_cursor(true).expect(errmsg);
        writeln!(stderr, "Something went wrong, I would regard this as an internal error.").expect(errmsg);
        writeln!(stderr, "{}", e.display_chain()).expect(errmsg);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let app = cli::get_app();
    let matches = app.get_matches();

    if let ("completions", Some(sub_matches)) = matches.subcommand() {
        let shell = sub_matches.value_of("SHELL").unwrap();
        cli::get_app().gen_completions_to(
            "todo",
            shell.parse().unwrap(),
            &mut std::io::stdout()
        );
        return Ok(());
    }

    let mut view = WindowView::new()?;
    view.set_cursor(false)?;
    ctrlc::set_handler(|| {}).chain_err(|| "Error setting Ctrl-C handler")?; // do explicitly nothing
    view.clear()?;

    let main_file_path = get_file_path()?;
    if !main_file_path.exists() {
        DirBuilder::new()
        .recursive(true)
        .create(&main_file_path)?;
    }

    match matches.subcommand() {
        ("open", Some(open_matches)) => Window::new_from_path(&open_matches.value_of("FILE").unwrap().to_string())?.run()?,
        ("init", Some(init_matches)) => {
            let mut path = canonicalize(PathBuf::from("./"))?;
            path.push(init_matches.value_of("FILE").unwrap());
            path.set_extension("todo");
            if path.exists() {
                println!("This would overwrite the file!  If you want to delete it either delete it manually or open it and use 'o'");
            } else {
                let list = todo_list::TodoList::create(path.file_stem().unwrap().to_str().unwrap().to_string(), path.to_str().unwrap().to_string());
                Window::new(WindowState::new(list))?.run()?;
            }
        },
        ("", None) => if let Some(list) = change_list()? {Window::new(WindowState::new(list))?.run()?;},
        _ => unreachable!(),
    }

    view.set_cursor(true)?;
    view.flush()?;
    Ok(())
}
