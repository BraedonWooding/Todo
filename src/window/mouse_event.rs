use std::ops::Rem;
use termion::event::{MouseButton, MouseEvent};
use window::*;

impl Window {
    pub fn handle_mouse_event(&mut self, event: MouseEvent) -> Result<bool> {
        let cur_list = self.state.cur_parent_list();
        if cur_list.len() == 0 {return Ok(false);}
        match event {
            MouseEvent::Press(MouseButton::WheelDown, _, _) => {
                // Better explanation
                if cur_list.len() > 0 {
                    let last = self.state.last_cur()?;
                    if *last == !0 {
                        *last = cur_list.len() - 1;
                    } else {
                        *last = ((*last as i64 - 1 + cur_list.len() as i64)
                            % (cur_list.len() as i64)) as usize;
                    }
                    self.dirty_window = true;
                }
            },
            MouseEvent::Press(MouseButton::WheelUp, _, _) => {
                if cur_list.len() > 0 {
                    let last = self.state.last_cur()?;
                    if *last == !0 {
                        *last = 0;
                    } else {
                        *last = (*last as u64 + 1).rem(cur_list.len() as u64) as usize;
                    }
                    self.dirty_window = true;
                }
            },
            _ => return Ok(true),
        }
        Ok(false)
    }
}