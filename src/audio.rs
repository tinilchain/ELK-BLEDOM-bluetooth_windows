use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SizedSample};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub struct AudioMonitor {
    volume: Arc<Mutex<f32>>,
    stream: Option<cpal::Stream>,
    volume_sender: mpsc::Sender<f32>,
}

impl AudioMonitor {
    pub fn new() -> (Self, mpsc::Receiver<f32>) {
        let (sender, receiver) = mpsc::channel();
        (
            AudioMonitor {
                volume: Arc::new(Mutex::new(0.0)),
                stream: None,
                volume_sender: sender,
            },
            receiver,
        )
    }

    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("无法找到默认输入设备");
        let config = device.default_input_config()?;

        let volume = Arc::clone(&self.volume);
        let sender = self.volume_sender.clone();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => self.create_stream::<f32>(&device, &config.into(), volume.clone(), sender.clone())?,
            cpal::SampleFormat::I16 => self.create_stream::<i16>(&device, &config.into(), volume.clone(), sender.clone())?,
            cpal::SampleFormat::U16 => self.create_stream::<u16>(&device, &config.into(), volume.clone(), sender.clone())?,
            _ => return Err("不支持的采样格式".into()),
        };

        stream.play()?;
        self.stream = Some(stream);
        Ok(())
    }

    fn create_stream<T>(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        volume: Arc<Mutex<f32>>,
        sender: mpsc::Sender<f32>,
    ) -> Result<cpal::Stream, Box<dyn std::error::Error>>
    where
        T: SizedSample + Sample<Float = f32>,
    {
        let err_fn = move |err: cpal::StreamError| eprintln!("音频流错误: {}", err);

        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let volume_value: f32 = data
                    .iter()
                    .map(|sample| {
                        let float_sample = sample.to_float_sample();
                        float_sample.abs()
                    })
                    .sum::<f32>() / data.len() as f32;

                *volume.lock().unwrap() = volume_value;
                sender.send(volume_value).unwrap_or_default();
            },
            err_fn,
            None,
        )?;

        Ok(stream)
    }

    pub fn stop(&mut self) {
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.lock().unwrap()
    }
}

impl Drop for AudioMonitor {
    fn drop(&mut self) {
        self.stop();
    }
} 