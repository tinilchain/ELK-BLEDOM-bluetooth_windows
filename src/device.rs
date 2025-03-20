use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Manager, Peripheral};
use chrono;
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use futures::executor::block_on;

pub struct Days {
    pub monday: u8,
    pub tuesday: u8,
    pub wednesday: u8,
    pub thursday: u8,
    pub friday: u8,
    pub saturday: u8,
    pub sunday: u8,
    pub all: u8,
    pub week_days: u8,
    pub weekend_days: u8,
    pub none: u8,
}

pub const WEEK_DAYS: Days = Days {
    monday: 0x01,
    tuesday: 0x02,
    wednesday: 0x04,
    thursday: 0x08,
    friday: 0x10,
    saturday: 0x20,
    sunday: 0x40,
    all: 0x01 + 0x02 + 0x04 + 0x08 + 0x10 + 0x20 + 0x40,
    week_days: 0x01 + 0x02 + 0x04 + 0x08 + 0x10,
    weekend_days: 0x20 + 0x40,
    none: 0x00,
};

pub struct Effects {
    pub jump_red_green_blue: u8,
    pub jump_red_green_blue_yellow_cyan_magenta_white: u8,
    pub crossfade_red: u8,
    pub crossfade_green: u8,
    pub crossfade_blue: u8,
    pub crossfade_yellow: u8,
    pub crossfade_cyan: u8,
    pub crossfade_magenta: u8,
    pub crossfade_white: u8,
    pub crossfade_red_green: u8,
    pub crossfade_red_blue: u8,
    pub crossfade_green_blue: u8,
    pub crossfade_red_green_blue: u8,
    pub crossfade_red_green_blue_yellow_cyan_magenta_white: u8,
    pub blink_red: u8,
    pub blink_green: u8,
    pub blink_blue: u8,
    pub blink_yellow: u8,
    pub blink_cyan: u8,
    pub blink_magenta: u8,
    pub blink_white: u8,
    pub blink_red_green_blue_yellow_cyan_magenta_white: u8,
}

pub const EFFECTS: Effects = Effects {
    jump_red_green_blue: 0x87,
    jump_red_green_blue_yellow_cyan_magenta_white: 0x88,
    crossfade_red: 0x8b,
    crossfade_green: 0x8c,
    crossfade_blue: 0x8d,
    crossfade_yellow: 0x8e,
    crossfade_cyan: 0x8f,
    crossfade_magenta: 0x90,
    crossfade_white: 0x91,
    crossfade_red_green: 0x92,
    crossfade_red_blue: 0x93,
    crossfade_green_blue: 0x94,
    crossfade_red_green_blue: 0x89,
    crossfade_red_green_blue_yellow_cyan_magenta_white: 0x8a,
    blink_red: 0x96,
    blink_green: 0x97,
    blink_blue: 0x98,
    blink_yellow: 0x99,
    blink_cyan: 0x9a,
    blink_magenta: 0x9b,
    blink_white: 0x9c,
    blink_red_green_blue_yellow_cyan_magenta_white: 0x95,
};

pub struct BleLedDevice {
    peripheral: Peripheral,
    characteristics: Vec<btleplug::api::Characteristic>,
}

impl BleLedDevice {
    pub fn new() -> BleLedDevice {
        block_on(async {
            let manager = Manager::new().await.unwrap();
            let adapters = manager.adapters().await.unwrap();
            let central = adapters.into_iter().nth(0).unwrap();
            let mut characteristics = Vec::new();
            let mut found_peripheral = None;

            central.start_scan(ScanFilter::default()).await.unwrap();
            thread::sleep(Duration::from_secs(2));

            let peripherals = central.peripherals().await.unwrap();
            
            println!("正在扫描蓝牙设备...");
            for p in peripherals {
                if let Ok(Some(props)) = p.properties().await {
                    if let Some(local_name) = props.local_name {
                        println!("发现设备: {}", local_name);
                        if local_name.contains("ELK-BLEDOM") {
                            println!("找到 ELK-BLEDOM 设备，正在连接...");
                            p.connect().await.unwrap();
                            thread::sleep(Duration::from_millis(500));
                            
                            println!("正在发现服务...");
                            p.discover_services().await.unwrap();
                            
                            let chars = p.characteristics();
                            println!("获取到特征值数量: {}", chars.len());
                            for c in chars.iter() {
                                println!("特征值 UUID: {}", c.uuid);
                                println!("特征值属性: {:?}", c.properties);
                                
                                if c.properties.contains(btleplug::api::CharPropFlags::WRITE_WITHOUT_RESPONSE) {
                                    characteristics.push(c.clone());
                                }
                            }
                            
                            if !characteristics.is_empty() {
                                println!("找到可写入的特征值数量: {}", characteristics.len());
                                found_peripheral = Some(p);
                                break;
                            }
                        }
                    }
                }
            }

            if let Some(peripheral) = found_peripheral {
                println!("设备连接成功！");
                let device = BleLedDevice {
                    peripheral,
                    characteristics,
                };

                // 初始化设备
                let char = device.get_characteristic();
                
                // 同步时间
                let system_time = chrono::offset::Local::now();
                device.peripheral
                    .write(
                        char,
                        &[
                            0x7e,
                            0x00,
                            0x83,
                            chrono::Timelike::hour(&system_time) as u8,
                            chrono::Timelike::minute(&system_time) as u8,
                            chrono::Timelike::second(&system_time) as u8,
                            chrono::Datelike::weekday(&system_time).number_from_monday() as u8,
                            0x00,
                            0xef,
                        ],
                        WriteType::WithoutResponse,
                    )
                    .await
                    .unwrap();

                // 开机
                device.peripheral
                    .write(
                        char,
                        &[0x7e, 0x00, 0x04, 0xf0, 0x00, 0x01, 0xff, 0x00, 0xef],
                        WriteType::WithoutResponse,
                    )
                    .await
                    .unwrap();

                println!("设置 LED 为红色闪烁模式...");
                
                // 设置亮度
                device.peripheral
                    .write(
                        char,
                        &[0x7e, 0x00, 0x01, 100, 0x00, 0x00, 0x00, 0x00, 0xef],
                        WriteType::WithoutResponse,
                    )
                    .await
                    .unwrap();

                // 设置颜色
                device.peripheral
                    .write(
                        char,
                        &[0x7e, 0x00, 0x05, 0x03, 255, 0, 0, 0x00, 0xef],
                        WriteType::WithoutResponse,
                    )
                    .await
                    .unwrap();

                thread::sleep(Duration::from_millis(100));

                // 设置闪烁效果
                device.peripheral
                    .write(
                        char,
                        &[0x7e, 0x00, 0x03, EFFECTS.blink_red, 0x03, 0x00, 0x00, 0x00, 0xef],
                        WriteType::WithoutResponse,
                    )
                    .await
                    .unwrap();

                // 设置闪烁速度
                device.peripheral
                    .write(
                        char,
                        &[0x7e, 0x00, 0x02, 50, 0x00, 0x00, 0x00, 0x00, 0xef],
                        WriteType::WithoutResponse,
                    )
                    .await
                    .unwrap();

                device
            } else {
                panic!("未找到匹配的设备！请确保 ELK-BLEDOM LED 控制器已开启并在范围内");
            }
        })
    }

    fn get_characteristic(&self) -> &btleplug::api::Characteristic {
        self.characteristics.get(0).unwrap()
    }

    pub async fn write_command(&self, command: &[u8]) {
        self.peripheral
            .write(
                self.get_characteristic(),
                command,
                WriteType::WithoutResponse,
            )
            .await
            .unwrap();
    }

    pub fn set_color(&self, red_value: u8, green_value: u8, blue_value: u8) {
        block_on(self.write_command(&[
            0x7e,
            0x00,
            0x05,
            0x03,
            red_value,
            green_value,
            blue_value,
            0x00,
            0xef,
        ]));
    }

    pub fn set_brightness(&self, value: u8) {
        block_on(self.write_command(&[
            0x7e,
            0x00,
            0x01,
            value.min(0x64),
            0x00,
            0x00,
            0x00,
            0x00,
            0xef,
        ]));
    }

    pub fn set_effect(&self, value: u8) {
        block_on(self.write_command(&[
            0x7e,
            0x00,
            0x03,
            value,
            0x03,
            0x00,
            0x00,
            0x00,
            0xef,
        ]));
    }

    pub fn set_effect_speed(&self, value: u8) {
        block_on(self.write_command(&[
            0x7e,
            0x00,
            0x02,
            value.min(0x64),
            0x00,
            0x00,
            0x00,
            0x00,
            0xef,
        ]));
    }

    pub fn power_on(&self) {
        block_on(self.write_command(&[
            0x7e,
            0x00,
            0x04,
            0xf0,
            0x00,
            0x01,
            0xff,
            0x00,
            0xef,
        ]));
    }

    pub fn power_off(&self) {
        block_on(self.write_command(&[
            0x7e,
            0x00,
            0x04,
            0x00,
            0x00,
            0x00,
            0xff,
            0x00,
            0xef,
        ]));
    }

    pub fn sync_time(&self) {
        let system_time = chrono::offset::Local::now();
        block_on(self.write_command(&[
            0x7e,
            0x00,
            0x83,
            chrono::Timelike::hour(&system_time) as u8,
            chrono::Timelike::minute(&system_time) as u8,
            chrono::Timelike::second(&system_time) as u8,
            chrono::Datelike::weekday(&system_time).number_from_monday() as u8,
            0x00,
            0xef,
        ]));
    }

    pub fn set_custom_time(&self, hour: u8, minute: u8, second: u8, day_of_week: u8) {
        block_on(self.write_command(&[
            0x7e,
            0x00,
            0x83,
            hour.min(23),
            minute.min(59),
            second.min(59),
            day_of_week.min(7).max(1),
            0x00,
            0xef,
        ]));
    }

    pub fn set_schedule_on(&self, days: u8, hours: u8, minutes: u8, enabled: bool) {
        let value = if enabled { days + 0x80 } else { days };
        block_on(self.write_command(&[
            0x7e,
            0x00,
            0x82,
            hours.min(23),
            minutes.min(59),
            0x00,
            0x00,
            value,
            0xef,
        ]));
    }

    pub fn set_schedule_off(&self, days: u8, hours: u8, minutes: u8, enabled: bool) {
        let value = if enabled { days + 0x80 } else { days };
        block_on(self.write_command(&[
            0x7e,
            0x00,
            0x82,
            hours.min(23),
            minutes.min(59),
            0x00,
            0x01,
            value,
            0xef,
        ]));
    }

    pub fn generic_command(&self, id: u8, sub_id: u8, arg1: u8, arg2: u8, arg3: u8) {
        block_on(self.write_command(&[
            0x7e,
            0x00,
            id,
            sub_id,
            arg1,
            arg2,
            arg3,
            0x00,
            0xef,
        ]));
    }
}
