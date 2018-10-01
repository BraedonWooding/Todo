use std::fs::{remove_file};

use dialoguer;
use termion::event::{Key};
use termion::input::{TermRead};

use todo_list;
use window::*;

impl Window {
    pub fn handle_key_event(&mut self, event: Key) -> Result<bool> {
        let cur_list = self.state.cur_parent_list();
        match event {
            Key::Down | Key::Char('j') => {
                self.state.move_cur_down(1)?;
                self.dirty_window = true;
            },
            Key::Up | Key::Char('k') => {
                self.state.move_cur_up(1)?;
                self.dirty_window = true;
            },
            Key::Left | Key::Char('h') => {
                if self.state.cur_depth() > 1 {
                    self.state.pop_cur()?;
                    self.dirty_window = true;
                }
            },
            Key::Right | Key::Char('l') if cur_list.len() > 0 => {
                let cur_item = self.state.cur_item()?;
                if cur_item.contents.len() > 0 {
                    self.state.push_cur(0);
                    self.dirty_window = true;
                }
            },
            Key::Char('\t') if cur_list.len() > 0 => {
                let cur_item = self.state.cur_item()?;
                if let Some(new_title) = self.view.get_user_input("New Item", false)? {
                    let new_item = todo_list::TodoItem::create(new_title);
                    let i = cur_item.contents.len();
                    cur_item.contents.insert(i, new_item);
                    self.state.push_cur(i);
                    self.state.changes = true;
                    self.dirty_window = true;
                }
            },
            Key::Char('E') if cur_list.len() > 0 => {
                let cur_item = self.state.cur_item()?;
                // edit from beginning
                if cur_list.len() > 0 {
                    if let Some(new_title) = self.view.get_user_input_buf("Edit Item", &cur_item.title, Some(0), false)? {
                        cur_item.title = new_title;
                        self.state.changes = true;
                    }
                }
            }
            Key::Char('e') if cur_list.len() > 0 => {
                let cur_item = self.state.cur_item()?;
                // edit from end
                if cur_list.len() > 0 {
                    if let Some(new_title) = self.view.get_user_input_buf("Edit Item", &cur_item.title, None, false)? {
                        cur_item.title = new_title;
                        self.state.changes = true;
                    }
                }
            },
            Key::Char('w') if cur_list.len() > 0 => {
                let cur_item = self.state.cur_item()?;
                // wipe line and edit
                if cur_list.len() > 0 {
                    if let Some(new_title) = self.view.get_user_input("Edit Item", false)? {
                        cur_item.title = new_title;
                        self.state.changes = true;
                    }
                }
            },
            Key::Char(' ') if cur_list.len() > 0 => {
                let cur_item = self.state.cur_item()?;
                // Toggle
                if cur_list.len() > 0 {
                    cur_item.ticked_off = !cur_item.ticked_off;
                    self.state.changes = true;
                }
            },
            Key::Char('i') => {
                // new item
                if let Some(new_title) = self.view.get_user_input("New Item", false)? {
                    let new_item = todo_list::TodoItem::create(new_title);
                    cur_list.insert(*self.state.last_cur()?, new_item);
                    self.state.changes = true;
                    self.dirty_window = true;
                }
            },
            Key::Char('a') => {
                // append new item
                if let Some(new_title) = self.view.get_user_input("New Item", false)? {
                    let new_item = todo_list::TodoItem::create(new_title);
                    {let last = self.state.last_cur()?;
                    *last = if *last + 1 <= cur_list.len() {*last + 1} else {*last};
                    cur_list.insert(*last, new_item);}
                    self.state.changes = true;
                    self.dirty_window = true;
                }
            },
            Key::Char('d') => {
                // delete item
                if cur_list.len() > 0 {
                    self.state.history = Some(cur_list.remove(*self.state.last_cur()?));
                    let last = self.state.last_cur()?.clone();
                    if last >= cur_list.len() {
                        if cur_list.len() == 0 {
                            if self.state.cur_depth() > 1 {
                                self.state.pop_cur()?;
                            }
                        } else {
                            *self.state.last_cur()? = cur_list.len() - 1;
                        }
                        self.dirty_window = true;
                    }
                    self.state.changes = true;
                }
            },
            Key::Char('u') => {
                if let Some(item) = self.state.history.clone() {
                    cur_list.insert(*self.state.last_cur()?, item);
                    self.state.changes = true;
                    self.dirty_window = true;
                }
                self.state.history = None;
            },
            Key::Char('g') => {
                if let Some(mut goto_loc) = self.view.get_user_input("Goto # (1-indexed): ", false)? {
                    // try to parse int
                    let last = self.state.last_cur()?;
                    if let Some(id) = goto_loc.find('-') {
                        goto_loc.remove(id);
                        match goto_loc.parse::<usize>() {
                            Ok(num) if num > 0 => *last = if num <= cur_list.len() {cur_list.len() - num} else {0},
                            _ => print!("\u{7}\u{7}"), // beep
                        }
                    } else {
                        match goto_loc.parse::<usize>() {
                            Ok(num) if num > 0 => *last = if num <= cur_list.len() {num - 1} else {cur_list.len() - 1},
                            _ => print!("\u{7}\u{7}"), // beep
                        }
                    }
                    self.dirty_window = true;
                }
            },
            Key::Char('K') => {
                // Move item up
                if cur_list.len() > 1 {
                    {
                    let last = self.state.last_cur()?;
                    if *last > 0 {
                        cur_list.swap(*last, *last - 1);
                        *last -= 1;
                    } else {
                        let item = cur_list.remove(*last);
                        *last = cur_list.len();
                        cur_list.insert(*last, item);
                    }
                    }
                    self.state.changes = true;
                    self.dirty_window = true;
                }
            },
            Key::Char('J') => {
                // Move item down
                if cur_list.len() > 1 {
                    {
                    let last = self.state.last_cur()?;
                    if *last < cur_list.len() - 1 {
                        cur_list.swap(*last, *last + 1);
                        *last += 1;
                    } else {
                        let item = cur_list.remove(*last);
                        cur_list.insert(0, item);
                        *last = 0;
                    }
                    }
                    self.state.changes = true;
                    self.dirty_window = true;
                }
            },
            Key::Char('H') => {
                if cur_list.len() > 0 {
                    let mut last = self.state.last_cur()?;
                    if self.state.cur_depth() > 1 && *self.state.cur(self.state.cur_depth() - 2)? > 0 {
                        let item = cur_list.remove(*last);
                        self.state.pop_cur()?;
                        let new_list = &mut self.state.cur_parent_list();
                        last = self.state.last_cur()?;
                        *last += 1;
                        new_list.insert(*last, item);
                    }
                    self.dirty_window = true;
                }
                self.state.changes = true;
            },
            Key::Char('L') => {
                if cur_list.len() > 0 {
                    let last = self.state.last_cur()?;
                    if *last > 0 {
                        let item = cur_list.remove(*last);
                        let new_list = &mut cur_list[*last - 1].contents;
                        *last -= 1;
                        let len = new_list.len();
                        new_list.insert(*self.state.push_cur(len), item);
                    }
                    self.dirty_window = true;
                }
                self.state.changes = true;
            },
            Key::PageDown => {
                if cur_list.len() > 0 {
                    let last = self.state.last_cur()?;
                    *last = cur_list.len() - 1;
                    self.dirty_window = true;
                }
            },
            Key::PageUp => {
                if cur_list.len() > 0 {
                    let last = self.state.last_cur()?;
                    *last = 0;
                    self.dirty_window = true;
                }
            },
            // System Commands
            Key::Ctrl('r') => {
                if dialoguer::Confirmation::new("Reset to disk?").interact()? {
                    self.state.reload_list()?;
                }
            },
            Key::Ctrl('s') => {
                self.state.save_list()?;
                self.state.changes = false;
                self.state.destructive_changes = false;
            },
            Key::Ctrl('S') => {
                if let Some(new_path) = self.view.get_user_input("Path to save to: ", false)? {
                    self.state.cur_loaded_list().path = new_path;
                    self.state.save_list()?;
                    self.state.changes = false;
                    self.state.destructive_changes = false;
                }
            },
            Key::Ctrl('c') => {
                if self.state.changes && dialoguer::Confirmation::new("Save current to disk?").interact()? {
                    self.state.save_list()?;
                }
                self.view.clear()?;
                if let Some(new_list) = super::change_list()? {
                    self.state.switch_list(new_list);
                } else {
                    bail!("Failed to switch list")
                }
            },
            Key::Ctrl('d') => {
                println!("If you change your mind you can always press 's' to resave this current list as it will just delete the file.\nBut as soon as you switch out the list (and choose not to save) or you quit it is gone forever!!!");
                if dialoguer::Confirmation::new("You sure you want to delete this list?").interact()? {
                    remove_file(&self.state.cur_loaded_list().path)?;
                    self.state.changes = true;
                    self.state.destructive_changes = true;
                }
            },
            Key::Ctrl('p') => {
                {
                let mut list = self.state.cur_loaded_list();
                if let Some(new_title) = self.view.get_user_input_buf("Edit Title", &list.name, None, false)? {
                    list.name = new_title;
                }
                }
                self.state.changes = true;
            },
            Key::Ctrl('h') => {
                self.view.clear()?;
                println!("{}", indoc!("
                Help Information for todo (Any key exists this view)\r
                - 'up' and 'k' / 'down' and 'j' arrow keys navigate the list verticaly\r
                - 'left' and 'h' / 'right' and 'l' navigate horizontally\r
                - 'tab' starts a new inner list\r
                - 'K'/'J' move the current item up/down\r
                - 'H'/'L' moves the current item out/in\r
                - 'g' allows you to go to a specific item (negative indexes go from end backwards)\r
                - 'd' deletes the current item and places it into buffer for 'u'\r
                - 'u' inserts currently deleted item\r
                - 'i' inserts a new item at the given index\r
                - 'a' appends a new item after the given index\r
                - 'space' will toggle the tick\r
                - 'e' edits the current item at the end of the buffer\r
                - 'E' edits the current item at the start\r
                - 'w' wipes the item before editing it\r
                - 'ctrl + c' changes the current todo list\r
                - 'ctrl + d' deletes the current list\r
                - 'ctrl + p' edits the current list title\r
                - 'ctrl + r' resets back to disk\r
                - 'ctrl + s' saves the current list\r
                - 'ctrl + S' save as\r
                - 'ctrl + h' shows this message\r
                - 'escape' or 'ctrl + q' exits the list\r"));
                ::std::io::stdin().keys().next();
            }
            _ => return Ok(true),
        }
        Ok(false)
    }
}