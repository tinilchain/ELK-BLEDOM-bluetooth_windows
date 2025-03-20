mod device;
use device::{BleLedDevice, EFFECTS};
use std::io::{self, Read};

fn main() {
    println!("正在连接 LED 控制器...");
    let device = BleLedDevice::new();
    
    println!("\n控制说明：");
    println!("- 按空格键：关闭 LED");
    println!("- 按回车键：开启红色闪烁效果");
    println!("- 按 q 键退出程序");
    
    let mut stdin = io::stdin();
    let mut buffer = [0; 1];
    
    loop {
        if let Ok(_) = stdin.read_exact(&mut buffer) {
            match buffer[0] {
                b' ' => {  // 空格键
                    println!("关闭 LED...");
                    device.power_off();
                }
                b'\r' | b'\n' => {  // 回车键
                    println!("设置红色闪烁效果...");
                    device.power_on();
                    device.set_brightness(100);
                    device.set_color(255, 0, 0);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    device.set_effect(device::EFFECTS.blink_red);
                    device.set_effect_speed(50);
                }
                b'q' | b'Q' => {  // q 键退出
                    println!("关闭 LED 并退出程序...");
                    device.power_off();
                    break;
                }
                _ => {}
            }
        }
    }
}
