use linux_embedded_hal::{i2cdev::core::I2CDevice, I2cdev};

use crate::walk_mod::WalkMod;


pub struct HyperMod{
    pub factory_name:String,
    pub mod_type:String,
    pub i2c_dev:I2cdev,
    pub func_mod:FunctionalMod
}
pub enum FunctionalMod{
    WalkMode(WalkMod)
}
fn vec_u8_to_ascii_string_lossy(vec: Vec<u8>) -> String {
    vec.into_iter()
       .filter_map(|byte| if byte <= 127 { Some(byte as char) } else { None })
       .collect()
}

pub trait IntoHyperMod {
    async fn to_hyper_mod(self)-> HyperMod;
    async fn read_factory_name(&mut self)->String;
    async fn read_mod_type_name(&mut self)->String;
}

impl IntoHyperMod for I2cdev{
    async fn to_hyper_mod(mut self)-> HyperMod{
        let factory_name = self.read_factory_name().await;
        let mod_type = self.read_mod_type_name().await;
        println!("{}",self.read_factory_name().await);
        match mod_type.as_str() {
            "s" =>{

            }
            _ =>{
                println!("unrecognized mod type {}",mod_type)
            }
        }
        HyperMod{
            factory_name,
            mod_type,
            i2c_dev: self,
            func_mod: FunctionalMod::WalkMode(WalkMod { coord_x: 0, coord_y: 0 ,is_sleeping: false }),
        }
    }
    async fn read_factory_name(&mut self) ->String{
        let rst_name_arr = self.smbus_read_i2c_block_data(0, 16);
        println!("{:?}",rst_name_arr);
        let name_arr = rst_name_arr.unwrap();
        println!("name_arr get {:?} is ascii {}",name_arr,name_arr.is_ascii());
        let converted_str = vec_u8_to_ascii_string_lossy(name_arr);
        println!("read name:{} successfully ",converted_str);
        converted_str
    }
    async fn read_mod_type_name(&mut self) ->String{
        let name_arr = self.smbus_read_i2c_block_data(16, 16).expect("读取 smbus失败");
        println!("name_arr get {:?} is ascii {}",name_arr,name_arr.is_ascii());
        let converted_str = vec_u8_to_ascii_string_lossy(name_arr);
        println!("read name:{} successfully ",converted_str);
        converted_str
    }
}