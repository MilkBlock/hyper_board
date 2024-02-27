mod command_args;
mod macros;
use std::{net::{TcpListener, TcpStream},  thread::sleep, time::Duration};

use clap::Parser;
use tungstenite::{accept, client::IntoClientRequest, stream::MaybeTlsStream, Message, WebSocket};

use crate::command_args::CommandArgs;

#[derive(Parser,Debug)]
#[command(author, version, about)]
pub struct Cli {
    ///设置文件地址
    #[arg(short, long)]
    wifi: String,
    #[arg(short, long)]
    ip: String,
    #[arg(short, long,default_value = "false")]
    local_test: bool,
}
fn main() {
    // let s = TcpStream::connect("127.0.0.1:11451").unwrap();
    let cli = Cli::parse();
    println!("{:?}",cli);
    let mut ws = if !cli.local_test{ tungstenite::connect("ws://1.13.2.149:11451").unwrap().0}
    else { tungstenite::connect("ws://127.0.0.1:11451").unwrap().0};
    // ws.send(Message::Text("hello server".to_string())).unwrap();
    ws.write(Message::Text("log:(This is board zynq 7020)".to_string()));
    ws.flush();
    ws.write(Message::Text(format!("register_board:({},{})",cli.wifi,cli.ip)));
    ws.flush();
    loop{
        let msg ={
            let mut msg = ws.read().unwrap();
            if let Message::Text(text) = msg {
                println!("receive \"{}\"",text);
                let cmd_args = CommandArgs::parse_command(text);
                match_command!(
                    command request_check with args () 
                    in cmd_args 
                    with sock ws
                )
            }
        };
    }
    ws.close(None).unwrap();
}
fn request_check(ws: &mut WebSocket<MaybeTlsStream<TcpStream>>){
    ws.write(Message::Text("check:()".to_string()));
    ws.flush();
}


