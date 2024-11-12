

use core::str;

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{List, ListItem, Paragraph},
    Frame,
};
use tokio_serial::SerialStream;

use crate::ui::Mode;
use crate::common::input::Input;

use super::layout::MyWidget;

pub struct RxTxWidget {
    receive_buf: Vec<String>,
    input:Input,
    hex_mode: bool,
    qa_mode: bool,
    enter_end:bool,
}

impl RxTxWidget {
    pub const fn new() -> Self {
        Self {
            receive_buf: vec![],
            input: Input::new(),
            hex_mode: false,
            qa_mode: false,
            enter_end:false,
        }
    }
}

impl MyWidget for RxTxWidget {
    fn event(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Char('h') => self.hex_mode = !self.hex_mode,
            KeyCode::Char('a') => self.qa_mode = !self.qa_mode,
            KeyCode::Char('n') => self.enter_end = !self.enter_end,
            _ => {}
        }
    }

    fn input(&mut self, key: &KeyEvent, serial:&mut SerialStream) {
        let mut read_buf = vec![0;4096]; 
        if let Ok(size) = serial.try_read(read_buf.as_mut_slice()){
            if let Ok(str) = str::from_utf8(&read_buf[..size].to_vec())  {
                self.receive_buf.push(str.to_string())
            }
        }


        match key.code {
            KeyCode::Char(c) => self.input.enter_char(c),
            KeyCode::Backspace => self.input.delete_char(),
            KeyCode::Left => self.input.move_cursor_left(),
            KeyCode::Right => self.input.move_cursor_right(),
            KeyCode::Enter => {
                serial.try_write(self.input.get_string().as_bytes()).unwrap();
                self.input.reset_cursor();
            },
            _ => {}
        }
    }

    fn build(&self, area: Rect, f: &mut Frame, mode: &Mode) {

        let [text_area, send_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        let list = self
            .receive_buf
            .iter()
            .map(|v| ListItem::new(Line::from(v.as_str())));
        f.render_widget(List::new(list), text_area);
        match mode {
            Mode::Command => {}
            Mode::Input => f.set_cursor(send_area.x + self.input.get_index() as u16 + 1, send_area.y),
        }
        f.render_widget(Paragraph::new(format!(">{}", self.input.get_string())), send_area);
    }

    fn state_list(&self) -> Vec<String> {
        return vec![
            format!("[{0}]Hex Mode(h)", if self.hex_mode { "x" } else { " " }),
            format!("[{0}]QA Mode(a)", if self.qa_mode { "x" } else { " " }),
            format!("[{0}]\\r\\n End(n)", if self.enter_end { "x" } else { " " }),
        ];
    }
}
