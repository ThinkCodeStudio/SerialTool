use index::IndexPage;
use layout::MainLayout;
use ratatui::{
    Frame, Terminal, backend::{Backend, CrosstermBackend}, crossterm::{
        event::{self, EnableMouseCapture, Event::{self, Key}, KeyCode, KeyEvent, KeyEventKind},
        execute,
        terminal::EnterAlternateScreen,
    }, layout::Rect
};
use serialport::{DataBits, FlowControl, Parity, StopBits};
use std::{any::Any, io, sync::{Arc, atomic::AtomicBool}, time::Duration};
use tokio::sync::Mutex;

pub trait UiWidget:Any {
    fn event(&mut self, key: &KeyEvent);
    fn build(&self, f: &mut Frame, area: Rect);
}

pub enum Mode {
    Command,
    Input,
}

pub enum Page {
    Index,
    Main,
    Exit,
}

pub enum Action {
    Input(KeyEvent),
    Data(Vec<u8>),
}

pub mod index;
pub mod layout;
pub mod rxtx;

#[derive(Clone)]
pub struct SerialInfo {
    path: String,
    baud_rate: u32,
    data_bits: DataBits,
    stop_bits: StopBits,
    parity: Parity,
    flow_control: FlowControl,
}

impl Default for SerialInfo {
    fn default() -> Self {
        Self {
            path: String::new(),
            baud_rate: 115200,
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            parity: Parity::None,
            flow_control: FlowControl::None,
        }
    }
}

pub struct UiApp {
    ui_widget: Box<dyn UiWidget>,
    exit_flag: AtomicBool,
    page: Page,
}

impl UiApp {
    pub fn new() -> Self {
        let serial_list = serialport::available_ports().expect("not found port path");
        Self {
            ui_widget: Box::new(IndexPage::new(serial_list)),
            exit_flag: AtomicBool::new(false),
            page: Page::Index,
        }
    }

    pub fn event_handler(&mut self){
        if let Ok(Event::Key(key)) = event::read() {
             if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter =>{
                        let ui_widget:&dyn Any = self.ui_widget.as_ref();
                        if let Some(index_page) = ui_widget.downcast_ref::<IndexPage>() {
                            let serial_info = index_page.get_serial_info();
                            self.ui_widget = Box::new(MainLayout::new(serial_info).expect("Failed to create MainLayout"));
                        }
                        else{
                            self.ui_widget.event(&key);
                        }
                    }
                    KeyCode::Esc => {
                        let ui_widget:&dyn Any = self.ui_widget.as_ref();
                        if ui_widget.downcast_ref::<MainLayout>().is_some() {
                            let serial_list = serialport::available_ports().expect("not found port path");
                            self.ui_widget = Box::new(IndexPage::new(serial_list));
                        }
                        else{
                            self.ui_widget.event(&key);
                        }
                    },
                    KeyCode::Char('q') => self.exit_flag.store(true, std::sync::atomic::Ordering::SeqCst),   
                    _=> self.ui_widget.event(&key),
                }
             }
        }
    }

    pub fn ui_show<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), String> {
        terminal.draw(|f| self.ui_widget.build(f, f.area())).unwrap();
        Ok(())
    }
}
