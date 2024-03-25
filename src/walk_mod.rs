use linux_embedded_hal::{i2cdev::core::I2CDevice, I2cdev};
pub struct WalkMod{
    pub coord_x:u8,
    pub coord_y:u8,
    pub is_sleeping:bool
}
impl WalkMod{
    pub fn send_handle_xy(&self,i2c_dev:&mut I2cdev){
        let data: [u8; 2] = [self.coord_x,self.coord_y];
        // let data_converted:&[u8]= bytemuck::cast_slice(&data);
        i2c_dev.smbus_write_block_data(40, &[1]).expect("smbus write失败");
        i2c_dev.smbus_write_block_data(50, &data).expect("smbus write失败");
        println!("send handle_xy {:?}",data);
    }
    pub fn send_is_sleeping(&self,i2c_dev:&mut I2cdev){
        let data: [bool; 1] = [self.is_sleeping];
        let data_converted:&[u8]= bytemuck::cast_slice(&data);
        i2c_dev.smbus_write_block_data(50, data_converted);
        println!("send is_sleeping {:?}",data_converted);
    }
}