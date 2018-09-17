use termion;
use termion::event::{Key};
use termion::{color, style};
use termion::input::{MouseTerminal, TermRead};
use termion::screen::AlternateScreen;
use termion::raw::{IntoRawMode, RawTerminal};
use std::io::{stdout, Write, Stdout, stdin};
use std::io::Result as IOResult;
use std;

use window_state::WindowState;
use errors::*;
use todo_list;

pub struct WindowView {
    out: MouseTerminal<AlternateScreen<RawTerminal<Stdout>>>,
    pub size: (u16, u16),
}

impl Write for WindowView {
    fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        self.out.write(buf)
    }

    fn flush(&mut self) -> IOResult<()> {
        self.out.flush()
    }
}

impl WindowView {
    pub fn new() -> Result<Self> {
        let size = termion::terminal_size()?;
        Ok(WindowView {
            out: MouseTerminal::from(AlternateScreen::from(stdout().into_raw_mode()?)),
            size: size,
        })
    }

    pub fn calc_size(&mut self) -> Result<()> {
        self.size = termion::terminal_size()?;
        Ok(())
    }

    fn get_color(val: usize) -> String {
        match val {
            0 ..= 33 => color::Fg(color::Red).to_string(),
            34 ..= 66 => color::Fg(color::Yellow).to_string(),
            _ => color::Fg(color::Green).to_string(),
        }
    }

    pub fn print_out_list(&mut self, win: &mut WindowState, offset: usize, mut amount: usize) -> Result<()> {
        self.clear()?;
        if amount == 0 { return Ok(()); }

        let cur_list = win.cur_loaded_list();
        let currently_ticked_off = cur_list.contents.iter().filter(|x| x.ticked_off).count();
        let percentage = if cur_list.contents.len() > 0 { (100 * currently_ticked_off) / cur_list.contents.len() } else { 0 };
        let title = format!("{bold}== {file}{flag1}{flag2} [{cur}/{total}, {color}{percentage}%{reset}{bold}] =={reset}", 
                        bold = style::Bold,
                        reset = style::Reset,
                        file = cur_list.name,
                        flag1 = if win.changes {"*"} else {""},
                        flag2 = if win.destructive_changes {"!"} else {""},
                        cur = currently_ticked_off,
                        total = cur_list.contents.len(),
                        color = Self::get_color(percentage),
                        percentage = percentage);
        // Technical Debt: This is hard coded, maybe make a method to figure this out by stripping ansi
        let w = self.size.0;
        write!(self, "{}", str::repeat(" ", (w as usize - (title.len() - 25)) / 2))?;
        write!(self, "{}\n\r", title)?;
        amount -= 1;

        for (i, item) in cur_list.contents.iter().skip(offset).enumerate() {
            if amount == 0 { break; }
            self.print_item(win, item, 0, *win.cur(0)? == i + offset, &mut amount)?;
        }
        Ok(())
    }

    fn print_sub_item(&mut self, item: &todo_list::TodoItem, at_pos: bool, depth: usize) -> Result<()> {
        write!(self,
            "{}{} [{}] {}\n\r",
            str::repeat("    ", depth),
            if at_pos {"→"} else {" "},
            if item.ticked_off {"✓"} else {" "},
            item.title,
        )?;
        Ok(())
    }

    // depth starts at 0
    fn print_item(&mut self, win: &WindowState, item: &todo_list::TodoItem, depth: usize, could_select: bool, amount: &mut usize) -> Result<()> {
        self.print_sub_item(item, could_select && win.cur_depth() == depth + 1, depth)?;
        *amount -= 1;
        for (i, child) in item.contents.iter().enumerate() {
            if *amount == 0 { break; }
            self.print_item(win, child, depth + 1, win.cur_depth() != depth + 1 && could_select && *win.cur(depth + 1)? == i, amount)?;
        }

        Ok(())
    }

    fn forward_word(pos: &mut usize, text: &String) {
        for ch in text.chars().skip(*pos + 1) {
            if ch == ' ' { break; }
            *pos += 1;
        }
    }

    fn backward_word(pos: &mut usize, text: &String) {
        for ch in text.chars().take(*pos) {
            if ch == ' ' { break; }
            *pos -= 1;
        }
    }

    fn split_path(path: &str) -> (Option<&str>, &str) {
        match path.rfind(std::path::is_separator) {
            Some(pos) => (Some(&path[..pos]), &path[pos + 1..]),
            None => (None, path)
        }
    }

    fn get_path_completion(buffer: &str) -> Vec<(String, String)> {
        let (base_dir, file) = Self::split_path(&buffer);
        let mut res = Vec::new();
        let lookup_dir = base_dir.unwrap_or(".");
        if let Ok(list) = std::fs::read_dir(lookup_dir) {
            for entry in list {
                if let Ok(entry) = entry {
                    let name = entry.file_name();
                    if let Ok(path) = name.into_string() {
                        if path.starts_with(file) {
                            let (name, display) = if let Some(dir) = base_dir {
                                (format!("{}{}{}", dir, std::path::MAIN_SEPARATOR, path),
                                    Some(path))
                            } else {
                                (path, None)
                            };

                            let is_dir = entry.metadata().ok()
                                            .map_or(false, |k| k.is_dir());
                            let suffix: String = (if is_dir {std::path::MAIN_SEPARATOR} else {' '}).to_string();
                            if let Some(display) = display {
                                res.push((name, suffix + &display));
                            }
                        }
                    }
                }
            }
        }
        res
    }

    pub fn get_user_input_buf(&mut self, prompt: &str, buf: &str, pos: Option<usize>, use_path: bool) -> Result<Option<String>> {
        self.set_cursor(true)?;
        self.flush()?;
        let mut buffer = String::new();
        buffer += buf;
        let mut old_buffer = String::new();
        let mut cur_pos = pos.unwrap_or(buf.len() - 1);
        let mut current_choices = vec![];
        let mut current_index = 0usize;
        let mut buffer_changed = true;

        write!(self, "{}: {}", prompt, buffer)?;
        self.flush()?;
        for c in stdin().keys() {
            match c {
                Ok(Key::Ctrl('c')) | Ok(Key::Ctrl('q')) => return Ok(None),
                Ok(Key::Char('\n')) => break,
                Ok(Key::Backspace) => {
                    if cur_pos <= buffer.len() - 1 {
                        buffer.remove(cur_pos);
                        if cur_pos == buffer.len() { cur_pos -= 1; }
                        buffer_changed = true;
                    }
                },
                Ok(Key::Left) => cur_pos -= 1,
                Ok(Key::Right) => cur_pos += 1,
                Ok(Key::Up) => Self::backward_word(&mut cur_pos, &buffer),
                Ok(Key::Down) => Self::forward_word(&mut cur_pos, &buffer),
                Ok(Key::Char('\t')) if use_path => {
                    if buffer_changed {
                        current_choices = Self::get_path_completion(&buffer);
                        current_index = 0;
                        old_buffer = buffer.clone();
                        buffer = current_choices[current_index].1.to_owned();
                    } else {
                        if current_index == !0 || current_index >= current_choices.len() - 1 {
                            current_index = 0;
                        } else {
                            current_index += 1;
                        }
                        buffer = current_choices[current_index].1.to_owned();
                    }
                },
                Ok(Key::Char(c)) => {
                    buffer.push(c);
                    cur_pos += 1;
                    buffer_changed = true;
                },
                Ok(Key::Esc) => {
                    if !buffer_changed {
                        buffer = old_buffer.clone();
                        buffer_changed = true;
                    }
                },
                _ => {},
            }
            write!(self, "{}{}{}: {}", termion::clear::CurrentLine, termion::cursor::Left(!0), prompt, buffer)?;
            self.flush()?;
        }

        self.set_cursor(false)?;
        self.flush()?;

        Ok(Some(buffer))
    }

    pub fn get_user_input(&mut self, prompt: &str, use_path: bool) -> Result<Option<String>> {
        self.get_user_input_buf(prompt, "", None, use_path)
    }

    pub fn clear(&mut self) -> Result<()> {
        write!(self, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1))?;
        Ok(())
    }

    pub fn set_cursor(&mut self, enabled: bool) -> Result<()> {
        if enabled {
            write!(self, "{}", termion::cursor::Show)?;
        } else {
            write!(self, "{}", termion::cursor::Hide)?;
        }
        Ok(())
    }
}
