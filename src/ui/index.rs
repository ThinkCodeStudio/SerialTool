use indoc::indoc;
use ratatui::{
    backend::Backend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        style::Color,
    },
    layout::{Constraint, Layout},
    style::Stylize,
    text::Line,
    widgets::Paragraph,
    Frame, Terminal,
};
use tokio_serial::{DataBits, FlowControl, Parity, SerialPortInfo, StopBits};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

use crate::ui::{AppContext, Page};

#[derive(PartialEq, Clone, Copy, Display, FromRepr, EnumIter)]
enum Menu {
    #[strum(to_string = "Serial Port")]
    SerialPort,
    #[strum(to_string = "Baud Rate")]
    BaudRate,
    #[strum(to_string = "Data Bits")]
    DataBits,
    #[strum(to_string = "Stop Bits")]
    StopBits,
    #[strum(to_string = "Parity")]
    Parity,
    #[strum(to_string = "Flow Conntrol")]
    FlowConntrol,
}

impl Menu {
    fn previous(self) -> Self {
        let current_index = self as usize;
        if current_index == 0 {
            return Menu::FlowConntrol;
        }
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap()
    }

    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(Menu::SerialPort)
    }
}

fn logo() -> String {
    let str = indoc! {r"
 ______     ______     ______     __     ______     __         ______   ______     ______     __        
/\  ___\   /\  ___\   /\  == \   /\ \   /\  __ \   /\ \       /\__  _\ /\  __ \   /\  __ \   /\ \       
\ \___  \  \ \  __\   \ \  __<   \ \ \  \ \  __ \  \ \ \____  \/_/\ \/ \ \ \/\ \  \ \ \/\ \  \ \ \____  
 \/\_____\  \ \_____\  \ \_\ \_\  \ \_\  \ \_\ \_\  \ \_____\    \ \_\  \ \_____\  \ \_____\  \ \_____\ 
  \/_____/   \/_____/   \/_/ /_/   \/_/   \/_/\/_/   \/_____/     \/_/   \/_____/   \/_____/   \/_____/ 
    "};
    return String::from(str);
}

const BAUD_RATE: [u32; 20] = [
    300, 600, 1200, 2400, 4800, 9600, 14400, 19200, 38400, 56000, 57600, 115200, 128000, 256000,
    460800, 512000, 750000, 900000, 921600, 1500000,
];

const DATA_BITS: [DataBits; 4] = [
    DataBits::Five,
    DataBits::Six,
    DataBits::Seven,
    DataBits::Eight,
];

const STOP_BITS: [StopBits; 2] = [StopBits::One, StopBits::Two];

const PARITY: [Parity; 3] = [Parity::None, Parity::Even, Parity::Odd];

const FLOW: [FlowControl; 3] = [
    FlowControl::None,
    FlowControl::Software,
    FlowControl::Hardware,
];

pub struct IndexPage {
    position: Menu,
    select: bool,
    index: usize,
    port_list: Vec<SerialPortInfo>,
}

impl IndexPage {
    pub const fn new(info: Vec<SerialPortInfo>) -> Self {
        return Self {
            position: Menu::SerialPort,
            select: false,
            index: 0,
            port_list: info,
        };
    }

    fn title(&self, position: Menu, value: &String) -> String {
        let mut text = if self.position == position && self.select {
            String::from("<")
        } else if self.position == position {
            String::from(">")
        } else {
            String::from(" ")
        };
        text.push_str(&position.to_string());
        text.push_str(": ");
        text.push_str(value.as_str());
        return text;
    }

    fn add_item(&self, text: &mut Vec<Line>, key: usize, value: &String) {
        let mut str = String::new();
        str.push_str("   ");
        str.push_str(key.to_string().as_str());
        str.push_str(": ");
        str.push_str(value.as_str());

        text.push(if self.index == key {
            Line::from(str).fg(Color::Yellow)
        } else {
            Line::from(str)
        });
    }

    fn add_number(&mut self, n: usize) {
        if self.index >= 10 {
            return;
        }
        self.index *= 10;
        self.index += n;
    }

    fn delete_number(&mut self) {
        self.index /= 10
    }

    fn down(&mut self) {
        self.position = self.position.next();
    }

    fn up(&mut self) {
        self.position = self.position.previous();
    }

    pub fn run<B: Backend>(
        &mut self,
        context: &mut AppContext,
        terminal: &mut Terminal<B>,
    ) -> Page {
        context.path = self.port_list[0].port_name.clone();
        loop {
            self.draw(context, terminal);
            if let Some(p) = self.event(context) {
                return p;
            }
        }
    }

    fn draw<B: Backend>(&self, context: &AppContext, terminal: &mut Terminal<B>) {
        terminal.draw(|f| self.build(context, f)).unwrap();
    }

    fn event(&mut self, context: &mut AppContext) -> Option<Page> {
        if let Ok(Event::Key(key)) = event::read() {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter => return Some(Page::Main),
                    KeyCode::Char('q') => return Some(Page::Exit),
                    KeyCode::Down => self.down(),
                    KeyCode::Up => self.up(),
                    KeyCode::Char('0') => {
                        if self.select {
                            self.add_number(0)
                        }
                    }
                    KeyCode::Char('1') => {
                        if self.select {
                            self.add_number(1)
                        }
                    }
                    KeyCode::Char('2') => {
                        if self.select {
                            self.add_number(2)
                        }
                    }
                    KeyCode::Char('3') => {
                        if self.select {
                            self.add_number(3)
                        }
                    }
                    KeyCode::Char('4') => {
                        if self.select {
                            self.add_number(4)
                        }
                    }
                    KeyCode::Char('5') => {
                        if self.select {
                            self.add_number(5)
                        }
                    }
                    KeyCode::Char('6') => {
                        if self.select {
                            self.add_number(6)
                        }
                    }
                    KeyCode::Char('7') => {
                        if self.select {
                            self.add_number(7)
                        }
                    }
                    KeyCode::Char('8') => {
                        if self.select {
                            self.add_number(8)
                        }
                    }
                    KeyCode::Char('9') => {
                        if self.select {
                            self.add_number(9)
                        }
                    }
                    KeyCode::Backspace => {
                        if self.select {
                            self.delete_number()
                        }
                    }
                    KeyCode::Right => self.select = true,
                    KeyCode::Left => {
                        self.select = false;
                        match self.position {
                            Menu::SerialPort => {
                                if self.index < self.port_list.len() {
                                    context.path = self.port_list[self.index].port_name.clone()
                                }
                            }

                            Menu::BaudRate => {
                                if self.index < BAUD_RATE.len() {
                                    context.baud_rate = BAUD_RATE[self.index]
                                }
                            }

                            Menu::DataBits => {
                                if self.index < DATA_BITS.len() {
                                    context.data_bits = DATA_BITS[self.index]
                                }
                            }

                            Menu::StopBits => {
                                if self.index < STOP_BITS.len() {
                                    context.stop_bits = STOP_BITS[self.index]
                                }
                            }

                            Menu::Parity => {
                                if self.index < PARITY.len() {
                                    context.parity = PARITY[self.index]
                                }
                            }

                            Menu::FlowConntrol => {
                                if self.index < FLOW.len() {
                                    context.flow_control = FLOW[self.index]
                                }
                            }
                        }
                        self.index = 0;
                    }
                    _ => {}
                }
            }
        }
        None
    }

    fn build(&self, context: &AppContext, f: &mut Frame) {
        let layout = Layout::vertical([
            Constraint::Percentage(10),
            Constraint::Percentage(90),
            // Constraint::Percentage(60),
        ])
        .split(f.size());
        let menu_layout =
            Layout::horizontal([Constraint::Percentage(33), Constraint::Percentage(67)])
                .split(layout[1]);

        let mut line_list = Vec::<Line>::new();
        let logo = logo();
        for v in logo.split('\n') {
            line_list.push(Line::from(v));
        }
        line_list.push(Line::from("-by ThinkCode").fg(Color::White));
        line_list.push(Line::from(""));
        line_list.push(
            Line::from(
                "*Press [Up&Down] to move, Press [Right] into item, Press [Left] back and save",
            )
            .fg(Color::White),
        );
        line_list.push(Line::from("*Press [Enter] to open Serial").fg(Color::Yellow));
        line_list.push(Line::from("*Press [q] to exit app").fg(Color::Red));
        line_list.push(Line::from(""));

        for menu in Menu::iter() {
            match menu {
                Menu::SerialPort => {
                    line_list.push(Line::from(self.title(menu, &context.path)));
                    if self.position == menu && self.select {
                        for (i, v) in self.port_list.iter().enumerate() {
                            self.add_item(&mut line_list, i, &v.port_name)
                        }
                    }
                }
                Menu::BaudRate => {
                    line_list.push(Line::from(self.title(menu, &context.baud_rate.to_string())));
                    if self.position == menu && self.select {
                        for (i, v) in BAUD_RATE.iter().enumerate() {
                            self.add_item(&mut line_list, i, &v.to_string())
                        }
                    }
                }
                Menu::DataBits => {
                    line_list.push(Line::from(self.title(menu, &context.data_bits.to_string())));
                    if self.position == menu && self.select {
                        for (i, v) in DATA_BITS.iter().enumerate() {
                            self.add_item(&mut line_list, i, &v.to_string())
                        }
                    }
                }
                Menu::StopBits => {
                    line_list.push(Line::from(self.title(menu, &context.stop_bits.to_string())));
                    if self.position == menu && self.select {
                        for (i, v) in STOP_BITS.iter().enumerate() {
                            self.add_item(&mut line_list, i, &v.to_string())
                        }
                    }
                }
                Menu::Parity => {
                    line_list.push(Line::from(self.title(menu, &context.parity.to_string())));
                    if self.position == menu && self.select {
                        for (i, v) in PARITY.iter().enumerate() {
                            self.add_item(&mut line_list, i, &v.to_string())
                        }
                    }
                }
                Menu::FlowConntrol => {
                    line_list.push(Line::from(
                        self.title(menu, &context.flow_control.to_string()),
                    ));
                    if self.position == menu && self.select {
                        for (i, v) in FLOW.iter().enumerate() {
                            self.add_item(&mut line_list, i, &v.to_string())
                        }
                    }
                }
            }
        }

        // f.render_widget(Paragraph::new(logo()).centered(), layout[1]);
        let paragraph = Paragraph::new(line_list).left_aligned();
        f.render_widget(paragraph, menu_layout[1]);
    }
}
