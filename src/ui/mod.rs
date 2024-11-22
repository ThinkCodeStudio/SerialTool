
use std::{io::Read, time::Duration};
use tokio::sync::mpsc::{self, Receiver, Sender};
use index::IndexPage;
use layout::MainLayout;
use ratatui::{backend::Backend, crossterm::event::KeyEvent, layout::Rect, Frame, Terminal};
use tokio_serial::{DataBits, FlowControl, Parity, SerialPort, SerialPortBuilderExt, SerialStream, StopBits};

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

    async fn serial_read(&self, serial_stream: SerialStream, tx_channel: Sender<Action>){
        let tx = tx_channel.clone();
        let serial_rx = serial_stream;
        tokio::spawn(async move {
            if let x   = serial_rx.read(buf).unwrap(){
                
            }
            tx
        });
    }
    
    pub async fn run_app<B: Backend>(&mut self, terminal:&mut Terminal<B>)->std::io::Result<()>{
        loop{
            match self.page {
                Page::Index => {
                    let serial_list = tokio_serial::available_ports().expect("not found port path");
                    self.path = serial_list[0].port_name.clone();
                    self.page = IndexPage::new(serial_list).run(self, terminal)
                },
                Page::Main => {
                    let (event_tx, event_rx) = mpsc::channel::<Action>(64);
                    let (send_tx, send_rx) = mpsc::channel::<Vec<u8>>(64);

                    let mut serial = tokio_serial::new(self.path.clone(), self.baud_rate)
                    .data_bits(self.data_bits)
                    .stop_bits(self.stop_bits)
                    .parity(self.parity)
                    .flow_control(self.flow_control)
                    .timeout(Duration::from_micros(1))
                    .open_native_async()
                    .expect(format!("open {} failed!", self.path).as_str());
                    self.serial_read(serial, event_tx);
                    self.page = MainLayout::default().run(terminal,&mut serial)
                },
                Page::Exit => {
                    return Ok(());
                },
            }
        }
    }
}