mod macros;
mod command_args;
mod request;
mod walk_mod;
mod hyper_mod;

use std::process::exit;
use std::vec;
use std::{fmt::format, process::Output, rc::Rc, sync::Arc};
use std::time::Duration;
use clap::Parser;
use linux_embedded_hal::i2cdev::core::I2CDevice;
use request::{read_server_sock, Stream};
use tokio::{net::TcpListener, sync::Mutex, time::{sleep, timeout},spawn};

use futures::stream::SplitSink;
use futures_util::{FutureExt, Sink, SinkExt, StreamExt};

use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, tungstenite::{self, handshake::server, Message, WebSocket}, WebSocketStream};
use tokio::{self, signal};
use linux_embedded_hal::{gpio_cdev::{self, Chip, Line, LineHandle, LineRequestFlags}, I2cdev};
use hyper_mod::{HyperMod, IntoHyperMod};

use crate::{command_args::CommandArgs, request::{log, read_app_sock, request_check, request_occupy, update_gpio, update_offset, ContextWithApp, ContextWithServer}};

#[derive(Parser,Debug)]
#[command(author, version, about)]
pub struct Args {
    ///设置文件地址
    #[arg(short, long)]
    wifi: String,
    #[arg(short, long)]
    ip: String,
    #[arg(short, long,default_value = "false")]
    local_test: bool,
    #[arg(short, long,default_value = "0")]
    gpiochip:u32,
    #[arg(long,default_value = "0")]
    output_gpio_left: u32,
    #[arg(long,default_value = "117")]
    output_gpio_right: u32,
    #[arg(short,long,default_value = "false")]
    test_mode_of_gpio: bool,
    #[arg(short,long,default_value = "false")]
    scan_gpio:bool,
    #[arg(short,long)]
    i2c_path:String,
}
pub struct GlobalData{
    ctrl_c_exited:bool
}
impl GlobalData{
    pub fn new()->Self{
        GlobalData{
            ctrl_c_exited:false
        }
    }
}
async fn check_ctrl_c(global_data_arctex:Arc<Mutex<GlobalData>>,board_data_arctex:Arc<Mutex<BoardData>>){
    signal::ctrl_c().await;
    global_data_arctex.lock().await.ctrl_c_exited = true;
    for gpio_handle in board_data_arctex.lock().await.output_gpio_handles.iter_mut(){
        if gpio_handle.set_value(0).is_err(){
            debug_info_red!("set value 0 失败 ")
        }
    }
    debug_info_green!("ctrl c 已经按下，设置所有 ouput gpio 为 0");
}
pub struct BoardData{
    occupied : bool,
    handle_x: f32,
    handle_y: f32,
    output_gpio_handles: Vec<LineHandle>,
    // walking_mod: ,
    hyper_mods:Vec<HyperMod>
}
impl BoardData{
    fn new() -> Self{
        BoardData{
            occupied:false,
            handle_x: 0.0,
            handle_y: 0.0,
            output_gpio_handles: vec![],
            hyper_mods: vec![],
        }
    }
}
async fn scan_gpio(board_data_arctex:Arc<Mutex<BoardData>>){
    for gpio in &mut board_data_arctex.lock().await.output_gpio_handles{
        println!("当前scan引脚为 {}",gpio.line().offset());
        gpio.set_value(1).expect(&format!("设置 {:?} 失败",gpio.line().offset()));
        sleep(Duration::from_secs_f32(0.5)).await;
        gpio.set_value(0).expect(&format!("设置 {:?} 失败",gpio.line().offset()));
        sleep(Duration::from_secs_f32(0.5)).await;
    }
}
async fn init_i2c(board_data_arctex: Arc<Mutex<BoardData>>,args:&Args){
    match I2cdev::new(args.i2c_path.clone()){
        Ok(mut i2cdev) =>{
            println!("")
        },
        Err(_) => println!("no i2c bus at {}",args.i2c_path),
    }
    
}
async fn scan_i2c(board_data_arctex: Arc<Mutex<BoardData>>,args:&Args){
    let addr_range = 0x40..=0x40;
    for i in addr_range{
        println!("try read {:x}",i);
        match I2cdev::new("/dev/i2c-1"){
            Ok(mut i2cdev) =>{
                match i2cdev.set_slave_address(i){
                    Ok(_) => {
                        let mut hyper_mod = i2cdev.to_hyper_mod().await;
                        board_data_arctex.lock().await.hyper_mods.push(hyper_mod)
                    },
                    Err(_) => println!("no dev at 0x{:x}",i),
                }
            },
            Err(_) => println!("no i2c bus at {}",args.i2c_path),
        }
    }
    let hyper_mods =&board_data_arctex.lock().await.hyper_mods;
    println!("collect {} hyper mods",hyper_mods.len());
    for hyper_mod in hyper_mods.iter(){
        println!("{}",hyper_mod.mod_type)
    }
}
#[tokio::main]
async fn main() {
    // let s = TcpStream::connect("127.0.0.1:11451").unwrap();
    let args = Args::parse();
    debug_info_blue!("{:?}",args);

    let board_data_arctex = Arc::new(Mutex::new(BoardData::new()));

    if !args.local_test {
        init_output_gpios(board_data_arctex.clone(),&args).await;
        if args.scan_gpio{
            scan_gpio(board_data_arctex.clone()).await;
            exit(0);
        }
        init_i2c(board_data_arctex.clone(), &args).await;
        scan_i2c(board_data_arctex.clone(), &args).await;
    }
    let global_data_arctex = Arc::new(Mutex::new(GlobalData::new()));

    debug_info_blue!("start server_oriented_task");
    let server = tokio::spawn(server_oriented_task(args.local_test, args.wifi, args.ip,board_data_arctex.clone(), global_data_arctex.clone()));
    debug_info_blue!("start app_oriented_task");
    spawn(app_oriented_task(board_data_arctex.clone(),global_data_arctex.clone())); 
    if args.test_mode_of_gpio {
        debug_info_blue!("start test_output_gpios");
        spawn(test_output_gpios(board_data_arctex.clone(),global_data_arctex.clone()));
    }
    spawn(check_ctrl_c(global_data_arctex.clone(), board_data_arctex.clone()));
    server.await;
}

async fn test_output_gpios(board_data_arctex: Arc<Mutex<BoardData>>,global_data_arctex:Arc<Mutex<GlobalData>>){
    debug_info_green!("启用 test 模式");
    loop{
        // 检测到 ctrl c 
        if global_data_arctex.lock().await.ctrl_c_exited == true{
            debug_info_yellow!("检测到 ctrl c, test_output_gpios 退出");
            break;
        }

        debug_info_blue!("set 0");
        for gpio in board_data_arctex.lock().await.output_gpio_handles.iter_mut() {
            if gpio.set_value(0).is_err(){ 
                debug_info_yellow!("set_gpio error in test task");
            };
        }
        sleep(Duration::from_secs_f32(0.5)).await;
        debug_info_blue!("set 1");
        for gpio in board_data_arctex.lock().await.output_gpio_handles.iter_mut() {
            if gpio.set_value(1).is_err(){ 
                debug_info_yellow!("set_gpio error in test task");
            };
        }
        sleep(Duration::from_secs_f32(0.5)).await;
    }
}

async fn init_output_gpios(board_data_arctex: Arc<Mutex<BoardData>>,cli:&Args){
    let chips:Vec<_> = gpio_cdev::chips().expect("没有任何chip ").collect();
    // I2cdev::new(path);
    debug_info_green!("available chips : {:?}",chips);
    let gpio_rst = Chip::new(format!("/dev/gpiochip{}",cli.gpiochip).as_str());
    
    if gpio_rst.is_err(){
        return
    }else{
        let mut gpio_chip = gpio_rst.unwrap();
        let output_gpios:Vec<u32> = (cli.output_gpio_left..cli.output_gpio_right).collect();
        let mut board_data =board_data_arctex.lock().await;
        for gpio in output_gpios{
            let rst_gpio_line = gpio_chip.get_line(gpio);
            if rst_gpio_line.is_err(){
                debug_info_red!("找不到对应的gpio {}",gpio);
            } else if let Ok(gpio_line) = rst_gpio_line{
                if let Ok(output_gpio_handle)= gpio_line.request(LineRequestFlags::OUTPUT, 1, "output of board"){
                    board_data.output_gpio_handles.push(output_gpio_handle);
                    debug_info_green!("gpio {} request成功, 新编号为 {}",gpio,board_data.output_gpio_handles.len()-1)
                }else {
                    debug_info_red!("gpio {} request失败",gpio)
                }

            }

        }
        // let gpio_lines = gpio_chip.get_lines(&output_gpios).expect("请求控制gpio 失败");// 目前有13 个gpio 引脚
        
    }
}


async fn server_oriented_task(is_local_test:bool,wifi:String,ip:String,board_data_arctex: Arc<Mutex<BoardData>>,global_data_arctex:Arc<Mutex<GlobalData>>){
    let (mut write,mut read) = if !is_local_test{ 
        debug_info_blue!("try connect remote server ");
        tokio_tungstenite::connect_async("ws://1.13.2.149:11451").await.unwrap().0.split()
    }
    else { 
        debug_info_blue!("try connect local server ");
        tokio_tungstenite::connect_async("ws://127.0.0.1:11451").await.unwrap().0.split()
    };
    // ws.send(Message::Text("hello server".to_string())).unwrap();
    write.send(Message::Text("log:(This is board zynq 7020)".to_string())).await;
    write.send(Message::Text(format!("register_board:({},{})",wifi,ip))).await;
    debug_info_blue!("sended");
    // 这个线程用于 告诉服务器自己在线
    let (write_arctex,read_arctex) = (Arc::new(Mutex::new(write)),Arc::new(Mutex::new(read)));

    let ctx = ContextWithServer{
        write: write_arctex,
        read: read_arctex,
        global_data: global_data_arctex.clone()
    };

    loop{
        // 检测到 ctrl c 
        if global_data_arctex.lock().await.ctrl_c_exited == true{
            debug_info_yellow!("检测到 ctrl c, server_oriented_task 退出");
            break;
        }
        let mut msg = read_server_sock(ctx.clone()).await;
        if let Message::Text(text) = msg {
            debug_info_green!("receive \"{}\"",text);
            let cmd_args = CommandArgs::parse_command(text);
            println!("server oriented receive");
            match_command!(
                command request_check with args () debug true
                in cmd_args 
                with context ctx
            )
        }
    }
    // ws.close(None).unwrap();
}


async fn app_oriented_task(board_data_arctex:Arc<Mutex<BoardData>>,global_data_arctex:Arc<Mutex<GlobalData>>){
    debug_info_green!("bind 端口11451 成功");
    let server = TcpListener::bind("0.0.0.0:11451").await.expect("bind 端口失败");

    // ? 服务端端口注册完毕，进行数据初始化

    while let (stream,_sock_addr) = server.accept().await.expect("accept 失败") {
        let cloned_board_data_arctex= board_data_arctex.clone();
        let cloned_global_data_arctex= global_data_arctex.clone();
        tokio::spawn(async move{
            // 初始化锁 
            let board_data_arctex = cloned_board_data_arctex;
            let global_data_arctex = cloned_global_data_arctex;
            debug_info_green!("connected app");
            let (write,read) = accept_async(stream).await.unwrap().split();
            let (write_arctex,read_arctex) = (Arc::new(Mutex::new(write)),Arc::new(Mutex::new(read)));
            let ctx = ContextWithApp{
                board_data_arctex: board_data_arctex,
                write: write_arctex,
                read: read_arctex,
                global_data_arctex:global_data_arctex.clone()
            };

            loop{
                if global_data_arctex.lock().await.ctrl_c_exited == true{
                    debug_info_yellow!("检测到 ctrl c, app_oriented_task 退出");
                    break;
                }
                let msg = read_app_sock(ctx.clone()).await;
                // if msg.is_close(){ close_sock( server_data_arctex).await; break; }
                if msg.is_text() {
                    let msg = msg.into_text().unwrap();
                    debug_info_green!("receive \"{}\"",msg);

                    let cmd_args = CommandArgs::parse_command(msg);
                    println!("server oriented receive");
                    match_command!(
                        command log with args (log_str:String) debug false
                        command request_occupy with args () debug true
                        command update_gpio with args (pin_num:usize,state:u8) debug true 
                        command update_offset with args (x:f32,y:f32) debug true
                        in cmd_args 
                        with context ctx
                    )
                }                
            }
        });
    }
}
