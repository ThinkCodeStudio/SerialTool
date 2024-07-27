use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{List, ListItem, Paragraph},
    Frame,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_serial::SerialStream;

use crate::ui::Mode;
use crate::common::input::Input;

use super::MyWidget;

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

    fn submit_message(&mut self) {
        self.receive_buf.push(self.input.get_string().clone());
        self.input.reset_cursor();
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

    fn input(&mut self, tx: &Sender<Vec<u8>>, key: &KeyEvent) {
        match key.code {
            KeyCode::Char(c) => self.input.enter_char(c),
            KeyCode::Backspace => self.input.delete_char(),
            KeyCode::Left => self.input.move_cursor_left(),
            KeyCode::Right => self.input.move_cursor_right(),
            KeyCode::Enter => self.submit_message(),
            _ => {}
        }
    }

    fn build(&self, area: Rect, f: &mut Frame, mode: &Mode, rx:&mut Receiver<Vec<u8>>) {
        let [text_area, send_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        if let Some(data) = rx.recv().await{

        }

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
            format!("[{0}]Hex Mode", if self.hex_mode { "x" } else { " " }),
            format!("[{0}]QA Mode", if self.qa_mode { "x" } else { " " }),
            format!("[{0}]\\r\\n End", if self.qa_mode { "x" } else { " " }),
        ];
    }
}
