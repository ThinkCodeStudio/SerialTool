use std::sync::Arc;

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Position, Rect},
    text::Line,
    widgets::{List, ListItem, Paragraph},
    Frame,
};
use serialport::SerialPort;
use tokio::sync::Mutex;

use crate::common::input::Input;
use crate::ui::Mode;

use super::layout::MyWidget;

pub struct RxTxWidget {
    receive_buf: Vec<String>,
    input: Input,
    hex_mode: bool,
    qa_mode: bool,
    enter_end: bool,
    serila: Arc<Mutex<Box<dyn SerialPort>>>
}

impl RxTxWidget {
    pub const fn new(serial: Arc<Mutex<Box<dyn SerialPort>>>) -> Self {
        Self {
            receive_buf: vec![],
            input: Input::new(),
            hex_mode: false,
            qa_mode: false,
            enter_end: false,
            serila: serial,
        }
    }

    fn serial_send(&self, serial: Arc<Mutex<Box<dyn SerialPort>>>, data: Vec<u8>) {
        tokio::spawn(async move {
            let mut serial = serial.lock().await;
            serial
                .write_all(&data)
                .expect("Failed to write to serial port");
        });
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

    fn input(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                if self.qa_mode{   
                    self.input.enter_char(c)
                }
                else{
                    self.serial_send(Arc::clone(&self.serila), vec![c as u8]);
                }
            },
            KeyCode::Backspace => self.input.delete_char(),
            KeyCode::Left => self.input.move_cursor_left(),
            KeyCode::Right => self.input.move_cursor_right(),
            KeyCode::Enter => {
                let enter_end = b"\r\n";
                if self.qa_mode {
                    let mut data = self.input.get_string().as_bytes().to_vec();
                    self.receive_buf.push(self.input.get_string().clone());
                    if self.enter_end {
                        data.extend_from_slice(enter_end);
                    }
                    self.serial_send(Arc::clone(&self.serila), data);
                    self.input.reset_cursor();
                }
                else{
                    self.serial_send(Arc::clone(&self.serila), enter_end.to_vec());
                }
            }
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
            Mode::Input => f.set_cursor_position(Position::new(
                send_area.x + self.input.get_index() as u16 + 1,
                send_area.y,
            )),
        }
        f.render_widget(
            Paragraph::new(format!(">{}", self.input.get_string())),
            send_area,
        );
    }

    fn state_list(&self) -> Vec<String> {
        return vec![
            format!("[{0}]Hex Mode(h)", if self.hex_mode { "x" } else { " " }),
            format!("[{0}]QA Mode(a)", if self.qa_mode { "x" } else { " " }),
            format!("[{0}]\\r\\n End(n)", if self.enter_end { "x" } else { " " }),
        ];
    }
}
