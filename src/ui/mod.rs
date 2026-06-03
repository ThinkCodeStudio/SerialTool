
use std::{sync::Arc, time::Duration};
use serialport::{DataBits, FlowControl, Parity, StopBits};
use index::IndexPage;
use layout::MainLayout;
use ratatui::{backend::Backend, crossterm::event::KeyEvent, Terminal};
use tokio::sync::Mutex;
pub enum Mode {
    Command,
    Input
}

pub enum Page {
    Index,
    Main,
    Exit
}

pub enum Action{
    Input(KeyEvent),
    Data(Vec<u8>)
}


pub mod index;
pub mod layout;
pub mod rxtx;

pub struct AppContext{
    path:String,
    baud_rate:u32,
    data_bits:DataBits,
    stop_bits:StopBits,
    parity:Parity,
    flow_control:FlowControl,
    page:Page
}

impl AppContext {

    pub const fn new()->Self{
        Self { 
            path: String::new(), 
            baud_rate: 115200, 
            data_bits: DataBits::Eight, 
            stop_bits: StopBits::One, 
            parity: Parity::None, 
            flow_control: FlowControl::None,
            page:Page::Index
        }
    }

    pub fn run_app<B: Backend>(&mut self, terminal:&mut Terminal<B>)->Result<(), String>{
        loop{
            match self.page {
                Page::Index => {
                    let serial_list = serialport::available_ports().expect("not found port path");
                    self.path = serial_list[0].port_name.clone();
                    self.page = IndexPage::new(serial_list).run(self, terminal)
                },
                Page::Main => {
                    match serialport::new(self.path.clone(), self.baud_rate)
                    .data_bits(self.data_bits)
                    .stop_bits(self.stop_bits)
                    .parity(self.parity)
                    .flow_control(self.flow_control)
                    .timeout(Duration::from_micros(1))
                    .open(){
                        Ok(serial) => {
                            self.page = MainLayout::new(Arc::new(Mutex::new(serial))).run(terminal);
                        },
                        Err(err) => {
                            return Err(format!("Failed to open serial port: {}, error: {}", self.path, err));
                        }
                    }
                },
                Page::Exit => {
                    return Ok(());
                },
            }
        }
    }
}