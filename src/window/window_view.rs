use termion;
use termion::event::{Key, Event, MouseButton, MouseEvent};
use termion::{color, clear, style};
use termion::input::{MouseTerminal, TermRead};
use termion::screen::AlternateScreen;
use termion::raw::{IntoRawMode, RawTerminal};
use std::io::{stdout, Write, Stdout};
use linefeed::{Interface, ReadResult, DefaultTerminal};
use linefeed::terminal::Signal;
use std::io::Result as IOResult;

use window_state::WindowState;
use errors::*;
use todo_list;
use window_state;

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

    pub fn get_user_input(&mut self, input_msg: &str, reader: &Interface<DefaultTerminal>) -> Result<ReadResult> {
        self.set_cursor(true)?;
        self.flush()?;
        let height = self.size.1;
        write!(self, "{}\r\n", termion::cursor::Goto(1, height - 2))?;
        reader.set_prompt(input_msg)?;
        reader.set_report_signal(Signal::Break, true);
        reader.set_report_signal(Signal::Interrupt, true);
        let result = reader.read_line().chain_err(|| "Reader Error");
        self.set_cursor(false)?;
        self.flush()?;
        result
    }

    pub fn get_user_input_buf(&mut self, input_msg: &str, buf: &str, pos: Option<usize>, reader: &Interface<DefaultTerminal>) -> Result<ReadResult> {
        reader.set_buffer(buf)?;
        if let Some(i) = pos {
            reader.set_cursor(i)?;
        }
        self.get_user_input(input_msg, reader)
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
