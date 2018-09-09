use std::ops::Rem;
use std::fs::{remove_file, canonicalize};
use std::path::{PathBuf};
use std::io::{Write, stdin, stdout, Read};

use dialoguer;
use linefeed::{Interface, ReadResult};
use termion;
use termion::event::{Key, Event, MouseButton, MouseEvent};
use termion::{color, style};
use termion::input::{TermRead};
use glob::glob;
use linefeed::complete::{PathCompleter, DummyCompleter};
use termion::raw::{IntoRawMode};
use std::sync::Arc;

use todo_list;
use errors::*;
use window_view::WindowView;
use window_state::WindowState;
use select_helper;
use util;

pub fn create_new_list(view: &mut WindowView) -> Result<Option<todo_list::TodoList>> {
    let reader = Interface::new("create new list")?;
    if let ReadResult::Input(list_name) = view.get_user_input("New List Name: ", &reader)? {
        let mut stdin = stdin();
        let mut view = WindowView::new()?;
        let list_prompt = format!("Where to place the {underline}list{reset}? ({bold}q{reset}/{bold}esc{reset} to exit)",
                                   bold = style::Bold, reset = style::Reset,
                                   underline = style::Underline);

        let choice = select_helper::select(&mut view, &mut stdin, list_prompt,
        &vec!["In current directory".to_owned(), "In home directory (~/_todo_lists/)".to_owned(), "Somewhere else".to_owned()])?;
        if choice.is_none() {
            view.set_cursor(true)?;
            return Ok(None);
        }
        
        let mut file_path = match choice.unwrap() {
            0 => PathBuf::from("./"),
            1 => util::get_file_path()?,
            2 => {
                reader.set_completer(Arc::new(PathCompleter));
                if let ReadResult::Input(dir) = view.get_user_input("Enter directory: ", &reader)? {
                    PathBuf::from(dir)
                } else {
                    view.set_cursor(true)?;
                    println!("\nInvalid Directory");
                    return Ok(None);
                }
            },
            _ => unreachable!(),
        };

        file_path = canonicalize(file_path)?;
        file_path.push(&list_name);
        file_path.set_extension("todo");
        Ok(Some(todo_list::TodoList::create(list_name, file_path.to_str().unwrap().to_string())))
    } else {
        view.set_cursor(true)?;
        println!("\nInvalid List Name");
        Ok(None)
    }
}

pub fn change_list() -> Result<Option<todo_list::TodoList>> {
    // Find all the lists and prompt user for which one
    let list;
    let mut possibilities = vec![];
    let mut stdin = stdin();
    let mut view = WindowView::new()?;
    for entry in glob(&(util::get_file_path()?.to_str().unwrap().to_string() + "/*.todo")).chain_err(|| "Can't change list")? {
        let entry = entry?;
        if entry.extension().unwrap() == "todo" {
            possibilities.push("~/_todo_lists/".to_owned() + &entry.file_stem().unwrap().to_str().unwrap().to_string());
        }
    }
    for entry in glob("./*.todo").chain_err(|| "Can't change list")? {
        let entry = entry?;
        if entry.extension().unwrap() == "todo" {
            possibilities.push("./".to_owned() + &entry.file_stem().unwrap().to_str().unwrap().to_string());
        }
    }
    possibilities.push("<New List>".to_owned());
    possibilities.push("<Open Other List>".to_owned());

    let list_prompt = format!("Where to place the {underline}list{reset}? ({bold}q{reset}/{bold}esc{reset} to exit)",
                                bold = style::Bold, reset = style::Reset,
                                underline = style::Underline);

    let choice = select_helper::select(&mut view, &mut stdin, list_prompt, &possibilities)?;

    if choice.is_none() {
        return Ok(None);
    }

    if choice.unwrap() == possibilities.len() - 2 {
        if let Some(new_list) = create_new_list(&mut view)? {
            list = new_list;
        } else {
            return Ok(None);
        }
    } else if choice.unwrap() == possibilities.len() - 1 {
        write!(view, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1))?;
        let reader = Interface::new("new directory")?;
        reader.set_completer(Arc::new(PathCompleter));
        if let ReadResult::Input(directory) = view.get_user_input_buf("Enter Directory: ", ::std::env::current_dir()?.to_str().unwrap(), None, &reader)? {
            list = WindowState::load_list(&directory.trim().to_string())?;
        } else {
            bail!("Invalid Directory")
        }
    } else {
        let full_path = possibilities[choice.unwrap()].clone() + ".todo";
        list = WindowState::load_list(&full_path)?;
    }
    Ok(Some(list))
}
