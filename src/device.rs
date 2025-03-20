use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Manager, Peripheral};
use chrono::{self, Timelike, Datelike};
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
    crossfade_red: 0x8B,
    crossfade_green: 0x8C,
    crossfade_blue: 0x8D,
    crossfade_yellow: 0x8E,
    crossfade_cyan: 0x8F,
    crossfade_magenta: 0x90,
    crossfade_white: 0x91,
    crossfade_red_green: 0x92,
    crossfade_red_blue: 0x93,
    crossfade_green_blue: 0x94,
    crossfade_red_green_blue: 0x89,
    crossfade_red_green_blue_yellow_cyan_magenta_white: 0x8A,
    blink_red: 0x95,
    blink_green: 0x96,
    blink_blue: 0x97,
    blink_yellow: 0x98,
    blink_cyan: 0x99,
    blink_magenta: 0x9A,
    blink_white: 0x9B,
    blink_red_green_blue_yellow_cyan_magenta_white: 0x9C,
};

pub struct BleLedDevice {
    peripheral: Peripheral,
    characteristics: Vec<btleplug::api::Characteristic>,
}

impl BleLedDevice {
    pub async fn connect(address: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().next().ok_or("No Bluetooth adapter found")?;

        central.start_scan(Default::default()).await?;
        std::thread::sleep(Duration::from_secs(2));

        let peripherals = central.peripherals().await?;
        let device = peripherals.into_iter()
            .find(|p| p.address().to_string() == address)
            .ok_or("Device not found")?;

        device.connect().await?;
        device.discover_services().await?;

        let characteristics = device.characteristics();
        let characteristics_vec = characteristics.into_iter().collect();
        Ok(Self { peripheral: device, characteristics: characteristics_vec })
    }

    fn get_characteristic(&self) -> &btleplug::api::Characteristic {
        &self.characteristics[0]
    }

    pub async fn write_command(&self, command: &[u8]) {
        if let Err(e) = self.peripheral
            .write(
                self.get_characteristic(),
                command,
                WriteType::WithoutResponse,
            )
            .await
        {
            eprintln!("写入命令失败: {}", e);
        }
    }

    pub fn set_color(&self, red_value: u8, green_value: u8, blue_value: u8) {
        let command = [0x7E, 0x00, 0x05, 0x03, red_value, green_value, blue_value, 0xEF];
        block_on(self.write_command(&command));
    }

    pub fn set_brightness(&self, value: u8) {
        let normalized_value = value.min(100);
        let command = [0x7E, 0x00, 0x01, normalized_value, 0x00, 0x00, 0x00, 0xEF];
        block_on(self.write_command(&command));
    }

    pub fn set_effect(&self, value: u8) {
        let command = [0x7E, 0x00, 0x03, value, 0x03, 0x00, 0x00, 0xEF];
        block_on(self.write_command(&command));
    }

    pub fn set_effect_speed(&self, value: u8) {
        let normalized_value = value.min(100);
        let command = [0x7E, 0x00, 0x02, normalized_value, 0x00, 0x00, 0x00, 0xEF];
        block_on(self.write_command(&command));
    }

    pub fn power_on(&self) {
        let command = [0x7E, 0x00, 0x04, 0x01, 0x00, 0x00, 0x00, 0xEF];
        block_on(self.write_command(&command));
    }

    pub fn power_off(&self) {
        let command = [0x7E, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0xEF];
        block_on(self.write_command(&command));
    }

    pub fn sync_time(&self) {
        let now = chrono::Local::now();
        self.set_custom_time(
            now.hour() as u8,
            now.minute() as u8,
            now.second() as u8,
            now.weekday().num_days_from_monday() as u8 + 1,
        );
    }

    pub fn set_custom_time(&self, hour: u8, minute: u8, second: u8, day_of_week: u8) {
        let command = [0x7E, 0x00, 0x83, hour, minute, second, day_of_week, 0xEF];
        block_on(self.write_command(&command));
    }

    pub fn set_schedule_on(&self, days: u8, hours: u8, minutes: u8, enabled: bool) {
        let command = [
            0x7E, 0x00, 0x82, days, hours, minutes,
            if enabled { 0x01 } else { 0x00 },
            0xEF
        ];
        block_on(self.write_command(&command));
    }

    pub fn set_schedule_off(&self, days: u8, hours: u8, minutes: u8, enabled: bool) {
        let command = [
            0x7E, 0x00, 0x82, days, hours, minutes,
            if enabled { 0x00 } else { 0x01 },
            0xEF
        ];
        block_on(self.write_command(&command));
    }

    pub fn generic_command(&self, id: u8, sub_id: u8, arg1: u8, arg2: u8, arg3: u8) {
        let command = [0x7E, id, sub_id, arg1, arg2, arg3, 0x00, 0xEF];
        block_on(self.write_command(&command));
    }
}
