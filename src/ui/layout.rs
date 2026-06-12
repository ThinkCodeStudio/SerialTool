use ratatui::{
    backend::Backend,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{Paragraph, Tabs},
    Frame, Terminal,
};
use serialport::SerialPort;
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use tokio::sync::Mutex;

use crate::ui::{Mode, Page, UiWidget};

use super::rxtx::RxTxWidget;

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
    layout: Box<dyn LayoutWidget>,
    serial: Arc<Mutex<Box<dyn SerialPort>>>,
}

impl MainLayout {
    pub fn new(serial_info: crate::ui::SerialInfo) -> Result<Self, String> {
        match serialport::new(serial_info.path.clone(), serial_info.baud_rate)
            .data_bits(serial_info.data_bits)
            .stop_bits(serial_info.stop_bits)
            .parity(serial_info.parity)
            .flow_control(serial_info.flow_control)
            .timeout(Duration::from_micros(1))
            .open()
        {
            Ok(serial) => Ok(Self {
                send_count: Default::default(),
                receive_count: Default::default(),
                selected_tab: Default::default(),
                mode: Mode::Command,
                serial: Arc::new(Mutex::new(serial)),
                layout: todo!(),
            }),
            Err(err) => {
                return Err(format!(
                    "Failed to open serial port: {}, error: {}", serial_info.path, err
                ));
            }
        }
    }
}

impl UiWidget for MainLayout {

    fn event(&mut self, key: &KeyEvent) {
        if key.kind == KeyEventKind::Press {
                match self.mode {
                    Mode::Command => match key.code {
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
                        _ => self.widget.input(&key),
                    },
                }
            }
    }

    fn build(&self, f: &mut Frame, area: Rect) {
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(f.area());

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

impl PageWidget for MainLayout {
        fn receive(&mut self, data: Vec<u8>) {
        self.receive_count += data.len();
    }
}