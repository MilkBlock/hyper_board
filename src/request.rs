use std::{fmt::format, process::Command, rc::Rc, sync::Arc, time::Duration};
use futures::stream::{SplitSink, SplitStream};
use tokio::{sync::Mutex, time::timeout};

use futures_util::{FutureExt, SinkExt, StreamExt};

use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::{ Message, WebSocket}, MaybeTlsStream, WebSocketStream};

use crate::{debug_info_blue, debug_info_red, hyper_mod::{self, FunctionalMod, HyperMod}, walk_mod, BoardData, GlobalData};


pub type Sink = SplitSink<WebSocketStream<TcpStream>,Message>;
pub type Stream = SplitStream<WebSocketStream<TcpStream>>;
pub type SinkArctex = Arc<Mutex<Sink>>;
pub type StreamArctex = Arc<Mutex<Stream>>;

// for server

#[derive(Clone)]
pub struct ContextWithServer{
    pub write:Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>,Message>>>,pub read:Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    pub global_data:Arc<Mutex<GlobalData>>
}

pub async fn request_check(ctx:ContextWithServer){
    ctx.write.lock().await.send(Message::Text("check:()".to_string())).await;
}

pub async fn read_server_sock(ctx:ContextWithServer)->Message{
    // debug_info!("{}","try read ");
    let cloned_read_arctex = ctx.read.clone();
    // 半小时之内没有通讯就删了你
    let msg = match timeout(Duration::from_secs(1800),cloned_read_arctex.lock().await.next()).await{
        Ok(Some(Ok(m)))=>m,
        Err(_e)=>{
            debug_info_red!("read 超时,断开连接");
            Message::Close(None)
        },
        _ => {
            Message::Close(None)
        }
    };
    msg
}

// for app 

#[derive(Clone)]
pub struct ContextWithApp{
    pub board_data_arctex:Arc<Mutex<BoardData>>,pub write:SinkArctex,pub read:StreamArctex,
    pub global_data_arctex:Arc<Mutex<GlobalData>>
}

pub async fn request_occupy(ctx:ContextWithApp){
    ctx.write.lock().await.send(Message::Text("response:(true)".to_string())).await;
}
pub async fn log(ctx:ContextWithApp,log_string:&String){
    debug_info_blue!("\x1B[34m{}\x1B[0m",log_string)
}

pub async fn update_offset(ctx:ContextWithApp,x:&f32,y:&f32){
    for hyper_mod in  ctx.board_data_arctex.lock().await.hyper_mods.iter_mut(){
        let u8_x: u8 = ((x+50.0) / 100.0 * 255.0) as u8; // 将-50-50映射到0-255
        let u8_y: u8 = ((y+50.0) / 100.0 * 255.0) as u8; // 将-50-50映射到0-255
        println!("{},{}",u8_x,u8_y);
        match &mut hyper_mod.func_mod {
            FunctionalMod::WalkMode(walk_mod) =>{
                walk_mod.coord_x=u8_x;
                walk_mod.coord_y=u8_y;
                walk_mod.send_handle_xy(&mut hyper_mod.i2c_dev)
            },
            // FunctionalMod::attacking_mod {  } => todo!(),
        }
    }
}

pub async fn update_gpio(ctx:ContextWithApp,offset:&usize,value:&u8){
    // for pin in &mut ctx.board_data_arctex.lock().await.output_gpio_pins{
    //     if pin.pin() ==  *num {
    //         if *y == 0 {
    //             pin.set_low()
    //         }else{
    //             pin.set_high()
    //         }
    //     }
    // }
    let handle = ctx.board_data_arctex.lock().await
        .output_gpio_handles.get_mut(*offset).unwrap().set_value(*value);
}

pub async fn read_app_sock(ctx:ContextWithApp)->Message{
    // debug_info!("{}","try read ");
    let cloned_read_arctex = ctx.read.clone();
    // 半小时之内没有通讯就删了你
    let msg = match timeout(Duration::from_secs(1800),cloned_read_arctex.lock().await.next()).await{
        Ok(Some(Ok(m)))=>m,
        Err(_e)=>{
            debug_info_red!("read 超时,断开连接");
            Message::Close(None)
        },
        _ => {
            Message::Close(None)
        }
    };
    msg
}