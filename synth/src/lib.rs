pub mod notes;
pub mod waves;

pub use crate::notes::Note;
use crate::waves::Wave;

use anyhow::{anyhow, Error, Result};
use core::hash::Hash;
pub use cpal::traits::StreamTrait;
use cpal::{
    traits::{DeviceTrait, HostTrait},
    BufferSize, Device, FromSample, OutputCallbackInfo, Sample, SampleFormat, SizedSample, Stream,
    StreamConfig, SupportedBufferSize, SupportedOutputConfigs,
};
use crossbeam_channel::Receiver;
use std::collections::HashSet;
use waves::Oscilator;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SupportedSampleFormat(SampleFormat);

impl From<SampleFormat> for SupportedSampleFormat {
    fn from(sample_format: SampleFormat) -> Self {
        Self(sample_format)
    }
}

impl Hash for SupportedSampleFormat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as usize).hash(state)
    }
}

#[derive(Clone, Debug, Default)]
pub struct SupportedConfigs {
    pub max_channels: u16,
    pub sample_rate_range: (u32, u32),
    pub buffer_size_range: Option<(u32, u32)>,
    pub supported_sample_formats: HashSet<SupportedSampleFormat>,
}

impl SupportedConfigs {
    fn new(configs: SupportedOutputConfigs) -> Self {
        let mut max_channels = 0;
        let mut sample_rate_range = (0, 0);
        let mut buffer_size_range = None;
        let mut supported_sample_formats = HashSet::new();
        configs.for_each(|config| {
            let channels = config.channels();
            let min_sample_rate = config.min_sample_rate();
            let max_sample_rate = config.max_sample_rate();
            let buffer_size = config.buffer_size();
            let sample_format = config.sample_format();

            max_channels = max_channels.max(channels);
            sample_rate_range = (min_sample_rate.0, max_sample_rate.0);
            match *buffer_size {
                SupportedBufferSize::Range { min, max } => buffer_size_range = Some((min, max)),
                SupportedBufferSize::Unknown => (),
            };

            supported_sample_formats.insert(sample_format.into());
        });

        Self {
            max_channels,
            sample_rate_range,
            buffer_size_range,
            supported_sample_formats,
        }
    }
}

pub struct AudioDevice {
    _input_device: Option<Device>,
    output_device: Device,
    pub supported_output_configs: SupportedConfigs,
}

impl AudioDevice {
    pub fn default() -> Result<Self> {
        let host = cpal::default_host();
        // println!("Host: {:?}", host.id().name());

        let input_device = host.default_input_device();
        // println!(
        //     "Input Device: {:?}",
        //     input_device
        //         .as_ref()
        //         .map(|device| device.name().unwrap_or("No Name".to_string()))
        // );

        let output_device = host
            .default_output_device()
            .ok_or(anyhow!("No default output device found."))?;
        // println!("Output Device: {:?}", output_device.name().ok());

        let supported_output_configs =
            SupportedConfigs::new(output_device.supported_output_configs()?);

        Ok(Self {
            output_device,
            _input_device: input_device,
            supported_output_configs,
        })
    }
}

pub struct Synth {
    device: AudioDevice,
    config: StreamConfig,
}

impl Synth {
    pub fn default() -> Result<Self> {
        let device = AudioDevice::default()?;
        Self::new(device)
    }

    pub fn new(device: AudioDevice) -> Result<Self> {
        let config = device.output_device.default_output_config()?;
        let config = config.into();
        Ok(Self { device, config })
    }

    pub fn channels(mut self, channels: u16) -> Result<Self> {
        let device_channels = self.device.supported_output_configs.max_channels;
        if channels < device_channels {
            self.config.channels = channels;
            Ok(self)
        } else {
            Err(anyhow!(
                "The device supports up to {device_channels} channels"
            ))
        }
    }

    pub fn sample_rate(mut self, sample_rate: u32) -> Result<Self> {
        let rate_range = self.device.supported_output_configs.sample_rate_range;
        if (rate_range.0..rate_range.1).contains(&sample_rate) {
            self.config.sample_rate.0 = sample_rate;
            Ok(self)
        } else {
            Err(anyhow!(
                "The device supports sample rates between {} and {}",
                rate_range.0,
                rate_range.1,
            ))
        }
    }

    pub fn buffer_size(mut self, buffer_size: u32) -> Result<Self> {
        let buffer_range = self
            .device
            .supported_output_configs
            .buffer_size_range
            .ok_or(anyhow!(
                "The accepted buffer sizes for the device are unknown"
            ))?;
        if (buffer_range.0..buffer_range.1).contains(&buffer_size) {
            self.config.buffer_size = BufferSize::Fixed(buffer_size);
            Ok(self)
        } else {
            Err(anyhow!(
                "The devices supports buffers size between {} and {}",
                buffer_range.0,
                buffer_range.1
            ))
        }
    }

    pub fn new_output_stream<T>(&mut self, wave: Wave) -> Result<Stream>
    where
        T: SizedSample + FromSample<f32>,
    {
        let channels = self.config.channels as u16;
        let sample_rate = self.config.sample_rate.0;
        let err_fn = |err| eprintln!("{}", err);

        let mut oscilator = Oscilator::new(sample_rate, wave);

        let data_callback = move |data: &mut [T], callback_info: &OutputCallbackInfo| {
            Self::write_data(data, &mut oscilator, channels, callback_info)
        };

        self.device
            .output_device
            .build_output_stream(&self.config, data_callback, err_fn, None)
            .map_err(Error::from)
    }

    pub fn new_output_stream_chan<T>(&mut self, wave: Wave, rx: Receiver<Wave>) -> Result<Stream>
    where
        T: SizedSample + FromSample<f32>,
    {
        let channels = self.config.channels as u16;
        let sample_rate = self.config.sample_rate.0;
        let err_fn = |err| eprintln!("{}", err);

        let mut oscilator = Oscilator::new(sample_rate, wave);
        oscilator.add_receiver(rx);

        let data_callback = move |data: &mut [T], callback_info: &OutputCallbackInfo| {
            Self::write_data(data, &mut oscilator, channels, callback_info)
        };

        self.device
            .output_device
            .build_output_stream(&self.config, data_callback, err_fn, None)
            .map_err(Error::from)
    }

    fn write_data<T>(
        output: &mut [T],
        oscilator: &mut Oscilator,
        channels: u16,
        _callback_info: &OutputCallbackInfo,
    ) where
        T: Sample + FromSample<f32>,
    {
        for frame in output.chunks_mut(channels as usize) {
            let value = oscilator.sample().to_sample();
            for sample in frame.iter_mut() {
                *sample = value;
            }
        }
    }
}
