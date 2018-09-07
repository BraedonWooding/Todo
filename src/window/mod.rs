use std::ops::Rem;
use std::fs::{remove_file, canonicalize};
use std::path::{PathBuf};
use std::io::{Write, stdin, stdout, Read};
use std::collections::HashMap;

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
use select_helper;
use util;

pub mod window_view;
pub mod window_state;
mod mouse_event;
mod key_event;

pub use self::window_view::*;
pub use self::window_state::*;

pub struct Window {
    state: WindowState,
    view: WindowView,
    dirty_window: bool,
}

impl Window {
    pub fn new(state: WindowState) -> Result<Self> {
        Ok(Window {
            state: state,
            view: WindowView::new()?,
            dirty_window: false,
        })
    }

    pub fn new_from_path(path: &String) -> Result<Self> {
        Self::new(WindowState::new_from_list(path)?)
    }

    fn calc_item_length(parent: &todo_list::TodoItem) -> usize {
        let mut sum = 1;
        for item in parent.contents.iter() {
            sum += Self::calc_item_length(item);
        }
        sum
    }

    pub fn run(&mut self) -> Result<()> {
        let stdin = stdin();
        let mut amount = self.view.size.1 as usize - 3;
        let mut offset = 0;
        self.view.print_out_list(&mut self.state, 0, amount)?;

        for c in stdin.events() {
            self.view.calc_size()?;
            let mut report_err;

            match c {
                Ok(Event::Mouse(mouse_event)) => report_err = self.handle_mouse_event(mouse_event)?,
                Ok(Event::Key(Key::Ctrl('q'))) | Ok(Event::Key(Key::Esc)) => {
                    if self.state.changes && dialoguer::Confirmation::new("Save current to disk?").interact()? {
                        self.state.save_list()?;
                    }
                    self.view.clear()?;
                    break;
                },
                Ok(Event::Key(key)) => report_err = self.handle_key_event(key)?,
                _ => report_err = true,
            }

            amount = self.view.size.1 as usize - 3;
            if self.dirty_window {
                offset = 0;
                let mut full_offset = 0;
                let parent = self.state.cur_loaded_list();
                let index = *self.state.first_cur()?;
                let list = &parent.contents;
                let mut relative = 0;
                while index > relative {
                    // refactor
                    let sum = Self::calc_item_length(&list[index - relative]);
                    if amount >= sum && full_offset < amount / 2 - sum - 2 {
                        full_offset += sum;
                    } else {
                        break;
                    }
                    relative += 1;
                    offset += 1;
                }
                if index <= relative {
                    offset = 0;
                } else if index > offset {
                    offset = index - offset;
                }
                self.dirty_window = false;
            }

            self.view.print_out_list(&mut self.state, offset, amount)?;
            self.view.set_cursor(false)?;
            self.view.flush()?;

            if report_err {
                write!(self.view, "\n\r{red}Unrecognised Key{reset}\n\r",
                    red = color::Fg(color::Red),
                    reset = style::Reset)?;
            }
        }
        Ok(())
    }
}

// pub fn manage_list(win: &mut WindowState) -> Result<()> {


//     // I know this is perfectly safe, but I can't convince Rust it is
//     // so I'm just gonna leave this here till I figure out how.


//         match c {
//             Ok(Event::Key(key)) => match key {
//                 Key::Down =>{
//                     if cur_list.len() > 0 {
//                         let last = self.state.last_cur()?;
//                         // Handles overflow
//                         // WHY am I doing this?? Stupid me made some weird code
//                         // this really needs to be done better...  honestly
//                         // get a grip!!!
//                         if *last == !0 {
//                             *last = 0;
//                         } else {
//                             *last = (*last as u64 + 1).rem(cur_list.len() as u64) as usize;
//                         }
//                     }
//                 },
//                 Key::Up =>{
//                     // Better explanation
//                     if cur_list.len() > 0 {
//                         let last = self.state.last_cur()?;
//                         if *last == !0 {
//                             *last = cur_list.len() - 1;
//                         } else {
//                             *last = ((*last as i64 - 1 + cur_list.len() as i64)
//                                 % (cur_list.len() as i64)) as usize;
//                         }
//                     }
//                 },
//                 Key::Left =>{
//                     if self.state.cur_depth() > 1 {
//                         self.state.pop_cur()?;
//                     }
//                 },
//                 Key::Right =>{
//                     // Go deeper
//                     let cur_item = &cur_list[*self.state.last_cur()?];
//                     if cur_item.contents.len() > 0 {
//                         self.state.push_cur(0);
//                     }
//                 }
//                 Key::Char('\t') => {

//                 },
//                 // Do I want the wrap-around?????
//                 Key::Char('k') => {
//                     // Move item up
//                     if cur_list.len() > 1 {
//                         {
//                         let last = self.state.last_cur()?;
//                         if *last > 0 {
//                             cur_list.swap(*last, *last - 1);
//                             *last -= 1;
//                         } else {
//                             // Query: Effectively this moves all the items down
//                             // could we maintain an offset into the list as to
//                             // make this significantly more efficient
//                             // I.e. O(n) => O(1), could also bubble swap
//                             // it all the way to the end, which arguably could
//                             // be more efficient???  In effect maybe list already
//                             // does this offset, else maybe we want to extend
//                             // with this offset??
//                             let item = cur_list.remove(*last);
//                             *last = cur_list.len();
//                             cur_list.insert(*last, item);
//                         }
//                         }
//                         self.state.changes = true;
//                     }
//                 },
//                 Key::Char('j') => {
//                     // Move item down
//                     if cur_list.len() > 1 {
//                         {
//                         let last = self.state.last_cur()?;
//                         if *last < cur_list.len() - 1 {
//                             cur_list.swap(*last, *last + 1);
//                             *last += 1;
//                         } else {
//                             let item = cur_list.remove(*last);
//                             cur_list.insert(0, item);
//                             *last = 0;
//                         }
//                         }
//                         self.state.changes = true;
//                     }
//                 },
//                 Key::Char('s') => {
//                     self.state.save_list()?;
//                     self.state.changes = false;
//                     self.state.destructive_changes = false;
//                 },
//                 Key::Char('P') => {
//                     // Not reasonable???
//                     if let ReadResult::Input(new_path) = view.get_user_input("Path to save to: ", &reader)? {
//                         let old_path = self.state.cur_loaded_list().path.clone();
//                         self.state.cur_loaded_list().path = new_path;
//                         self.state.save_list()?;
//                         remove_file(old_path)?;
//                         // Now remove old file
//                         self.state.changes = false;
//                         self.state.destructive_changes = false;
//                     }
//                 },
//                 Key::Char('S') => {
//                     if let ReadResult::Input(new_path) = view.get_user_input("Path to save to: ", &reader)? {
//                         self.state.cur_loaded_list().path = new_path;
//                         self.state.save_list()?;
//                         self.state.changes = false;
//                         self.state.destructive_changes = false;
//                     }
//                 },
//                 Key::Char('E') => {
//                     // Edit from beginning
//                     if cur_list.len() > 0 {
//                         let cur_item = &mut cur_list[*self.state.last_cur()?];
//                         if let ReadResult::Input(new_title) = view.get_user_input_buf("Edit Item: ", &cur_item.title, Some(0), &reader)? {
//                             cur_item.title = new_title;
//                             self.state.changes = true;
//                         }
//                     }
//                 },
//                 Key::Char('e') => {
//                     // Edit
//                     if cur_list.len() > 0 {
//                         let cur_item = &mut cur_list[*self.state.last_cur()?];
//                         if let ReadResult::Input(new_title) = view.get_user_input_buf("Edit Item: ", &cur_item.title, None, &reader)? {
//                             cur_item.title = new_title;
//                             self.state.changes = true;
//                         }
//                     }
//                 },
//                 Key::Char(' ') => {
//                     // Toggle
//                     if cur_list.len() > 0 {
//                         let cur_item = &mut cur_list[*self.state.last_cur()?];
//                         cur_item.ticked_off = !cur_item.ticked_off;
//                         self.state.changes = true;
//                     }
//                 },
//                 Key::Char('n') => {
//                     // new item
//                     if let ReadResult::Input(new_title) = view.get_user_input("New Item: ", &reader)? {
//                         let new_item = todo_list::TodoItem::create(new_title);
//                         cur_list.insert(*self.state.last_cur()?, new_item);
//                         self.state.changes = true;
//                     }
//                 },
//                 Key::Char('d') => {
//                     // delete item
//                     if cur_list.len() > 0 {
//                         self.state.history = Some(cur_list.remove(*self.state.last_cur()?));
//                         let last = self.state.last_cur()?.clone();
//                         if last >= cur_list.len() {
//                             if cur_list.len() == 0 {
//                                 if self.state.cur_depth() > 1 {
//                                     self.state.pop_cur()?;
//                                 }
//                             } else {
//                                 *self.state.last_cur()? = cur_list.len() - 1;
//                             }
//                         }
//                         self.state.changes = true;
//                     }
//                 },
//                 Key::Char('u') => {
//                     // if let Some(item) = *self.state.history {
//                     //     cur_list.insert(*self.state.last_cur()?, item);
//                     //     self.state.changes = true;
//                     // }
//                     // self.state.history = None;
//                 },
//                 Key::Char('r') => {
//                     if dialoguer::Confirmation::new("Reset to disk?").interact()? {
//                         self.state.reload_list()?;
//                     }
//                 },
//                 Key::Char('w') => {
//                     if cur_list.len() > 0 {
//                         let cur_item = &mut cur_list[*self.state.last_cur()?];
//                         if let ReadResult::Input(new_title) = view.get_user_input("Edit Item: ", &reader)? {
//                             cur_item.title = new_title;
//                             self.state.changes = true;
//                         }
//                     }
//                 },
//                 Key::Char('a') => {
//                     // append new item
//                     if let ReadResult::Input(new_title) = view.get_user_input("New Item: ", &reader)? {
//                         let new_item = todo_list::TodoItem::create(new_title);
//                         {let last = self.state.last_cur()?;
//                         *last = if *last + 1 <= cur_list.len() {*last + 1} else {*last};
//                         cur_list.insert(*last, new_item);}
//                         self.state.changes = true;
//                     }
//                 },
//                 Key::Char('p') => {
//                     {
//                     let mut list = self.state.cur_loaded_list();
//                     if let ReadResult::Input(new_title) = view.get_user_input_buf("Edit Title: ", &list.name, None, &reader)? {
//                         list.name = new_title;
//                     }
//                     }
//                     self.state.changes = true;
//                 },
//                 Key::Char('g') => {
//                     if let ReadResult::Input(mut goto_loc) = view.get_user_input("Goto # (1-indexed): ", &reader)? {
//                         // try to parse int
//                         let last = self.state.last_cur()?;
//                         if let Some(id) = goto_loc.find('-') {
//                             goto_loc.remove(id);
//                             match goto_loc.parse::<usize>() {
//                                 Ok(num) if num > 0 => *last = if num <= cur_list.len() {cur_list.len() - num} else {0},
//                                 _ => print!("\u{7}\u{7}"), // beep
//                             }
//                         } else {
//                             match goto_loc.parse::<usize>() {
//                                 Ok(num) if num > 0 => *last = if num <= cur_list.len() {num - 1} else {cur_list.len() - 1},
//                                 _ => print!("\u{7}\u{7}"), // beep
//                             }
//                         }
//                     }
//                 },
//                 Key::Char('b') => {
//                     if cur_list.len() > 0 {
//                         let last = self.state.last_cur()?;
//                         if self.state.cur_depth() > 1 && *self.state.cur(self.state.cur_depth() - 2)? > 0 {
//                             let cur_item = cur_list.remove(*last);
//                             self.state.pop_cur()?;
//                             let new_list = &mut self.state.cur_parent_list();
//                             *last += 1;
//                             new_list.insert(*last, cur_item);
//                         }
//                     }
//                     self.state.changes = true;
//                 },
//                 Key::Char('v') => {
//                     if cur_list.len() > 0 {
//                         let last = self.state.last_cur()?;
//                         if *last > 0 {
//                             let cur_item = cur_list.remove(*last);
//                             let new_list = &mut cur_list[*last - 1].contents;
//                             *last -= 1;
//                             let len = new_list.len();
//                             new_list.insert(*self.state.push_cur(len), cur_item);
//                         }
//                     }
//                     self.state.changes = true;
//                 },
//                 Key::Char('h') => {
//                     // Help
//                     view.clear(&mut view)?;
//                     println!("{}", indoc!(r#"
//                     Help Information for todo (Any key exists this view)
//                     - 'up'/'down' arrow keys navigate the list
//                     - 'left'/'right' goes into/out of inner lists
//                     - 'tab' starts a new inner list
//                     - 'k'/'j' move the current item up/down
//                     - 'b'/'v' moves the current item out/in
//                     - 'r' resets back to disk
//                     - 's' saves the current list
//                     - 'n' adds a new item
//                     - 'a' appends item to end
//                     - 'g' to goto a specific index
//                     - 'S' save as
//                     - 'P' change path (move)
//                     - 'd' deletes an item and places it into a buffer
//                     - 'u' will place whatever item is in the buffer at the current cursor position
//                     - 'h' shows this message
//                     - 'space' toggles the 'tick' Rc
//                     - 'e' edits the current item
//                     - 'c' changes the current todo list
//                     - 'o' deletes the current todo list
//                     - 'p' changes the list name
//                     - 'escape'/'q' exits and optionally saves the list
//                     - 'w' same as 'e' but wipes the text to allow you to rename from the beginning"#));
//                     ::std::io::stdin().keys().next();
//                 },
//                 Key::Char('o') => {
//                     // Set options
//                 },
//                 Key::Esc | Key::Char('q') => {
//                     if self.state.changes && dialoguer::Confirmation::new("Save current to disk?").interact()? {
//                         self.state.save_list()?;
//                     }
//                     view.clear(&mut view)?;
//                     break;
//                 },
//                 Key::Char('c') => {
//                     if self.state.changes && dialoguer::Confirmation::new("Save current to disk?").interact()? {
//                         self.state.save_list()?;
//                     }
//                     view.clear(&mut view)?;
//                     if let Some(new_list) = super::change_list()? {
//                         self.state.switch_list(new_list);
//                     } else {
//                         return Ok(())
//                     }
//                 },
//                 Key::Char('D') => {
//                     println!("If you change your mind you can always press 's' to resave this current list as it will just delete the file.\nBut as soon as you switch out the list (and choose not to save) or you quit it is gone forever!!!");
//                     if dialoguer::Confirmation::new("You sure you want to delete this list?").interact()? {
//                         remove_file(&self.state.cur_loaded_list().path)?;
//                         self.state.changes = true;
//                         self.state.destructive_changes = true;
//                     }
//                 }
//                 _ => report_err = true,
//             },

//         }
//     }
//     Ok(())
// }