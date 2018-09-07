// A very simple sub library to implement selection

use termion;
use termion::event::{Key, Event, MouseButton, MouseEvent};
use termion::{color, style};
use termion::input::{TermRead};
use std::io::{Write, Stdin};
use std::iter;
use std::slice;
use errors::*;
use window::WindowView;

const SCROLL_FACTOR: usize = 1;

fn print_out_selections(view: &mut WindowView, prompt: &String, options: iter::Enumerate<slice::Iter<String>>, cur: usize) -> Result<()> {
    write!(view, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1))?;
    view.write(prompt.as_bytes())?;
    view.write(b"\n\r")?;
    for (i, option) in options {
        write!(view, "{} {}\n\r", if i == cur {"→"} else {" "}, option)?;
    }
    view.flush()?;
    Ok(())
}

// Ignores any Ctrl + C or whatever
// Purely a normal terminal read.
pub fn select(view: &mut WindowView, stdin: &mut Stdin, prompt: String, options: &Vec<String>) -> Result<Option<usize>> {
    let mut cur = 0;
    write!(view, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1))?;
    print_out_selections(view, &prompt, options.iter().enumerate(), cur)?;

    for c in stdin.events() {
        let mut report_err = false;

        match c {
            Ok(Event::Key(key)) => match key {
                Key::Char('q') | Key::Esc => return Ok(None),
                Key::Up => cur = if cur > 0 {cur - 1} else {options.len() - 1},
                Key::Down => cur = if cur < options.len() - 1 {cur + 1} else {0},
                Key::Char('\n') => break,
                _ => report_err = true,
            },
            Ok(Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _))) => {
                let new_cur = cur + SCROLL_FACTOR;
                cur = if new_cur < options.len() - 1 {new_cur} else {options.len() - 1};
            },
            Ok(Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _))) => {
                cur = if cur < SCROLL_FACTOR {0} else {cur - SCROLL_FACTOR};
            }
            _ => report_err = true,
        }
        print_out_selections(view, &prompt, options.iter().enumerate(), cur)?;
        if report_err {
            write!(view, "\n\r{red}Unrecognised Key{reset}\n\r",
                red = color::Fg(color::Red),
                reset = style::Reset)?;
        }
    }
    Ok(Some(cur))
}