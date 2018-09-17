use std::io::{Write, stdin};

use termion::event::{Key, Event};
use termion::{color, style};
use termion::input::{TermRead};

use todo_list;
use errors::*;
use dialoguer;

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
        let mut amount = self.view.size.1 as usize - 4;
        let mut offset = 0;
        self.view.print_out_list(&mut self.state, 0, amount)?;

        for c in stdin().events() {
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

            amount = self.view.size.1 as usize - 4;
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
                write!(self.view, "\r\n{red}Unrecognised Key{reset}\r\n",
                    red = color::Fg(color::Red),
                    reset = style::Reset)?;
            }
        }
        Ok(())
    }
}
