use ratatui::{
    backend::Backend,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{Paragraph, Tabs},
    Frame, Terminal,
};
use strum::IntoEnumIterator;

use strum::{Display, EnumIter, FromRepr};
use tokio_serial::SerialStream;

use crate::ui::{Mode, Page};

use super::rxtx::RxTxWidget;

pub trait MyWidget {
    fn event(&mut self, key: &KeyEvent);
    fn input(&mut self, key: &KeyEvent, serial: &mut SerialStream);
    fn build(&self, area: Rect, f: &mut Frame, mode: &Mode);
    fn state_list(&self) -> Vec<String>;
}

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
pub enum SelectedTab {
    #[default]
    #[strum(to_string = "TxRx(t)")]
    TxRx,
    #[strum(to_string = "List(l)")]
    Command,
    #[strum(to_string = "Stream(s)")]
    Stream,
    #[strum(to_string = "Ymodem(y)")]
    Ymodem,
    #[strum(to_string = "Chart(c)")]
    Chart,
}

impl SelectedTab {
    fn title(self) -> Line<'static> {
        format!("  {self}  ").into()
    }

    fn previous(self) -> Self {
        let current_index = self as usize;
        if current_index == 0 {
            return SelectedTab::Chart;
        }
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap()
    }

    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(SelectedTab::TxRx)
    }
}

pub struct MainLayout {
    send_count: usize,
    receive_count: usize,
    selected_tab: SelectedTab,
    mode: Mode,

    widget: Box<dyn MyWidget>,
}

impl Default for MainLayout {
    fn default() -> Self {
        Self {
            send_count: Default::default(),
            receive_count: Default::default(),
            selected_tab: Default::default(),
            mode: Mode::Command,
            widget: Box::new(RxTxWidget::new()),
        }
    }
}

impl MainLayout {
    pub fn run<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        serial: &mut SerialStream,
    ) -> Page {
        loop {
            self.draw(terminal, serial);
            if let Some(page) = self.event(serial) {
                return page;
            }
        }
    }

    fn draw<B: Backend>(&self, terminal: &mut Terminal<B>, serial: &mut SerialStream) {
        terminal.draw(|f| self.build(f, serial)).unwrap();
    }

    fn event(&mut self, serial: &mut SerialStream) -> Option<Page> {
        if let Ok(Event::Key(key)) = event::read() {
            if key.kind == KeyEventKind::Press {
                match self.mode {
                    Mode::Command => match key.code {
                        KeyCode::Esc => return Some(Page::Index),
                        KeyCode::Char('q') => return Some(Page::Exit),
                        KeyCode::Char('t') => self.selected_tab = SelectedTab::TxRx,
                        KeyCode::Char('l') => self.selected_tab = SelectedTab::Command,
                        KeyCode::Char('s') => self.selected_tab = SelectedTab::Stream,
                        KeyCode::Char('y') => self.selected_tab = SelectedTab::Ymodem,
                        KeyCode::Char('c') => self.selected_tab = SelectedTab::Chart,
                        KeyCode::Char('i') => self.mode = Mode::Input,
                        KeyCode::Right => self.selected_tab = self.selected_tab.next(),
                        KeyCode::Left => self.selected_tab = self.selected_tab.previous(),
                        _ => self.widget.event(&key),
                    },
                    Mode::Input => match key.code {
                        KeyCode::Esc => self.mode = Mode::Command,
                        _ => self.widget.input(&key, serial),
                    },
                }
            }
        }
        None
    }

    fn read(&self, serial: &mut SerialStream) -> Option<Vec<u8>> {
        let mut read_buf = vec![0; 4096];
        if let Ok(size) = serial.try_read(read_buf.as_mut_slice()) {
            return Some(read_buf[..size].to_vec());
        }
        return None;
    }

    fn build(&self, f: &mut Frame, serial: &mut SerialStream) {
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(f.size());

        let [tab_area, text_area] =
            Layout::horizontal([Constraint::Percentage(80), Constraint::Percentage(20)])
                .areas(layout[0]);

        let state_layout = Layout::horizontal([
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ])
        .split(layout[2]);

        let titles = SelectedTab::iter().map(SelectedTab::title);

        let mut state_tabs = vec![
            String::from(format!("send:{0}", self.send_count)),
            String::from(format!("receive:{0}", self.receive_count)),
        ];

        for v in self.widget.state_list().iter() {
            state_tabs.push(v.clone());
        }

        f.render_widget(
            Tabs::new(titles).select(self.selected_tab as usize),
            tab_area,
        );
        f.render_widget(
            Paragraph::new("[i] input mode | [q] exit app | [Esc] back"),
            text_area,
        );
        self.widget.build(layout[1], f, &self.mode);
        for (i, v) in state_tabs.iter().enumerate() {
            f.render_widget(Paragraph::new(v.clone()), state_layout[i]);
        }
    }
}
