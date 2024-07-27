
use std::time::Duration;
use tokio::sync::mpsc::{self, Receiver, Sender};
use index::IndexPage;
use layout::MainLayout;
use ratatui::{backend::Backend, crossterm::event::KeyEvent, layout::Rect, Frame, Terminal};
use tokio_serial::{DataBits, FlowControl, Parity, SerialPortBuilderExt, StopBits};

pub enum Mode {
    Command,
    Input
}

pub enum Page {
    Index,
    Main,
    Exit
}


pub mod index;
pub mod layout;
pub mod rxtx;

pub trait MyWidget {
    fn event(&mut self,key: &KeyEvent);
    fn input(&mut self, tx: &Sender<Vec<u8>>, key: &KeyEvent);
    fn build(&self, area: Rect, f: &mut Frame, mode:&Mode, rx:&mut Receiver<Vec<u8>>);
    fn state_list(&self) -> Vec<String>;
}

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
    
    pub fn run_app<B: Backend>(&mut self, terminal:&mut Terminal<B>)->std::io::Result<()>{
        loop{
            match self.page {
                Page::Index => {
                    let serial_list = tokio_serial::available_ports().expect("not found port path");
                    self.path = serial_list[0].port_name.clone();
                    self.page = IndexPage::new(serial_list).run(self, terminal)
                },
                Page::Main => {
                    let mut serial = tokio_serial::new(self.path.clone(), self.baud_rate)
                    .data_bits(self.data_bits)
                    .stop_bits(self.stop_bits)
                    .parity(self.parity)
                    .flow_control(self.flow_control)
                    .timeout(Duration::from_micros(1))
                    .open_native_async()
                    .expect(format!("open {} failed!", self.path).as_str());

                    let (rx_sender, mut rx_receive) = mpsc::channel::<Vec<u8>>(1024);
                    let (tx_sender, mut tx_receive) = mpsc::channel::<Vec<u8>>(1024);
                    tokio::spawn(async move {
                        let mut buff = vec![0;4096];
                        loop{
                            if let Ok(size) = serial.try_read(buff.as_mut_slice()){   
                                rx_sender.send(buff[..size].to_vec());
                            }
                            if let Some(data) = tx_receive.recv().await{   
                                serial.try_write(data.as_slice());
                            }
                        }
                    });

                    self.page = MainLayout::default().run(tx_sender, terminal, &mut rx_receive)
                },
                Page::Exit => {
                    return Ok(());
                },
            }
        }
    }
}