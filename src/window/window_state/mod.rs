use std::io::{stdout, Write, Stdout};
use std::fs::{File, read_to_string};

use toml;
use termion;
use termion::event::{Key, Event, MouseButton, MouseEvent};
use termion::{color, clear, style};
use termion::input::{MouseTerminal, TermRead};
use termion::screen::AlternateScreen;
use termion::raw::{IntoRawMode, RawTerminal};
use std::cell::RefCell;

use errors::*;

use todo_list;
mod helper;
pub use self::helper::*;

pub struct WindowState {
    pub changes: bool, // lines have changed
    pub destructive_changes: bool, // i.e. File has been removed
    cur: RefCell<Vec<RefCell<usize>>>,
    loaded_lists: Vec<RefCell<todo_list::TodoList>>,
    cur_list: usize,
    // Note: this is bad, this should really be a list of changes
    // and we should be able to revert, I'm just lazy rn
    // and this is a pretty big undertaking to do properly.
    pub history: Option<todo_list::TodoItem>,
}

impl WindowState {
    pub fn new(list: todo_list::TodoList) -> Self {
        WindowState {
            changes: false,
            destructive_changes: false,
            cur: RefCell::from(vec![RefCell::from(0)]),
            loaded_lists: vec![RefCell::from(list)],
            cur_list: 0,
            history: None,
        }
    }

    pub fn new_from_list(path: &String) -> Result<Self> {
        Ok(Self::new(Self::load_list(path)?))
    }

    pub fn load_list(path: &String) -> Result<todo_list::TodoList> {
        let mut list : todo_list::TodoList = toml::from_str(&read_to_string(path)?).chain_err(|| "Failed to load list")?;
        list.path = path.to_owned();
        Ok(list)
    }

    // TODO: update this to use an index and a list
    pub fn switch_list(&mut self, list: todo_list::TodoList) {
        if let Some(pos) = self.loaded_lists.iter().position(|ref r| r.borrow_mut().path == list.path) {
            self.reset_cur();
            self.history = None;
            self.changes = false;
            self.destructive_changes = false;
            self.cur_list = pos;
        } else {
            self.reset_cur();
            self.history = None;
            self.changes = false;
            self.destructive_changes = false;
            self.cur_list = self.loaded_lists.len();
            self.loaded_lists.push(RefCell::from(list));
        }
    }

    pub fn reload_list(&self) -> Result<()> {
        let list = self.cur_loaded_list();
        list.contents = Self::load_list(&list.path)?.contents;
        Ok(())
    }

    pub fn cur_loaded_list(&self) -> &mut todo_list::TodoList {
        let res = self.loaded_lists[self.cur_list].as_ptr();
        unsafe {&mut *res}
    }

    pub fn save_list(&self) -> Result<()> {
        let list = self.cur_loaded_list();
        let mut file = File::create(&list.path)?;
        file.write_all(&toml::to_vec(&list)?)?;
        Ok(())
    }

    pub fn has_items(&self) -> bool {
        self.cur_loaded_list().contents.len() > 0
    }

    // Note: this function currently avoids the borrowing rules
    // by using an unsafe block and pointer dereferencing, please note
    // that by asking for a parent list you do not borrow that list
    // and merely maintain a reference to that list.
    // NOTE: I feel like a recursive solution is the way to solve this
    //       but I also feel like that'll be much slower and could cause problems
    //       on large lists, time for benchmarking!!
    pub fn cur_parent_list<'a>(&self) -> &'a mut Vec<todo_list::TodoItem> {
        let mut it = &mut self.cur_loaded_list().contents as *mut Vec<todo_list::TodoItem>;
        unsafe {
            for i in 0 .. self.cur.borrow().len() - 1 {
                let tmp = self.cur.borrow();
                it = &mut (*it)[*tmp[i].borrow()].contents as *mut Vec<todo_list::TodoItem>;
            }
            &mut *it
        }
    }

    pub fn cur_item<'a>(&self) -> Result<&'a mut todo_list::TodoItem> {
        Ok(&mut self.cur_parent_list()[*self.last_cur()?])
    }

    pub fn reset_cur(&self) {
        self.cur.borrow_mut().clear();
        self.cur.borrow_mut().push(RefCell::from(0));
    }

    pub fn cur_depth(&self) -> usize {
        self.cur.borrow().len()
    }

    pub fn move_cur_down(&self, amount: usize) -> Result<()> {
        let list_len = self.cur_parent_list().len();
        if list_len > 0 {
            let last = self.last_cur()?;
            // handling overflow as well
            if amount <= !0 && *last <= !0 - amount && *last < self.cur_parent_list().len() - 1 {
                *last += amount;
            } else {
                *last = 0;
            }
        }
        Ok(())
    }

    pub fn move_cur_up(&self, amount: usize) -> Result<()> {
        let list_len = self.cur_parent_list().len();
        if list_len > 0 {
            let last = self.last_cur()?;
            // handling underflow
            if *last >= amount {
                *last -= amount;
            } else  {
                *last = list_len - 1;
            }
        }
        Ok(())
    }

    pub fn set_cur(&self, values: &[usize]) {
        self.reset_cur();
        for val in values {
            self.push_cur(*val);
        }
    }

    pub fn pop_cur(&self) -> Result<usize> {
        if self.cur.borrow().len() <= 1 { bail!("Can't pop nothing or the last element") }
        let tmp = self.cur.borrow_mut().pop().unwrap();
        let res = tmp.borrow().clone();
        Ok(res)
    }

    pub fn push_cur(&self, new: usize) -> &mut usize {
        self.cur.borrow_mut().push(RefCell::from(new));
        self.last_cur().unwrap()
    }

    pub fn last_cur(&self) -> Result<&mut usize> {
        let len = self.cur_depth() - 1;
        self.cur(len)
    }

    pub fn first_cur(&self) -> Result<&mut usize> {
        self.cur(0)
    }

    pub fn cur(&self, depth: usize) -> Result<&mut usize> {
        if self.cur.borrow().len() <= depth { bail!("Depth out of range") }
        let res = self.cur.borrow_mut()[depth].as_ptr();
        unsafe {Ok(&mut *res)}
    }
}
