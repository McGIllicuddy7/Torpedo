use std::{
    f32::consts::E,
    sync::{Mutex, atomic::AtomicU8},
    time::Duration,
};

use cpal::traits::{DeviceTrait, HostTrait};

pub fn init_audio() -> cpal::Stream {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");
    let supported_stream_config = supported_configs_range
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    let config = supported_stream_config.config();
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _data| {
                AUDIO.read(data);
            },
            move |_err| {},
            Some(Duration::from_millis(60)),
        )
        .unwrap();
    stream
}

pub const RATE: usize = 44000;
pub const BUFFER_TIME: usize = 600;
pub struct AudioStream {
    pub data: [AtomicU8; RATE * BUFFER_TIME],
    pub ptr: Mutex<usize>,
}

pub static AUDIO: AudioStream = AudioStream {
    data: [const { AtomicU8::new(0) }; _],
    ptr: Mutex::new(0),
};
impl AudioStream {
    pub fn write(&self, bytes: &[u8]) {
        let base = *self.ptr.lock().unwrap();
        for i in 0..bytes.len() {
            let pt = (base + i) % (RATE * BUFFER_TIME);
            let mut tmp = self.data[pt].load(std::sync::atomic::Ordering::SeqCst);
            let mut m = std::cmp::max(tmp, bytes[i]);
            while let Err(t) = self.data[pt].compare_exchange_weak(
                tmp,
                m,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            ) {
                tmp = t;
                m = std::cmp::max(tmp, bytes[i]);
            }
        }
    }

    pub fn read(&self, bytes: &mut [f32]) {
        let mut ptr = self.ptr.lock().unwrap();
        for i in 0..bytes.len() {
            bytes[i] = self.data[*ptr].load(std::sync::atomic::Ordering::SeqCst) as f32 / 256.;
            self.data[*ptr].store(0, std::sync::atomic::Ordering::SeqCst);
            *ptr += 1;
            *ptr %= RATE * BUFFER_TIME;
        }
    }

    pub fn write_byte_func(&self, mut bytes: impl FnMut(f32) -> u8, time: f32) {
        let base = *self.ptr.lock().unwrap();
        let count = (time * RATE as f32) as usize;
        for i in 0..count {
            let pt = (base + i) % (RATE * BUFFER_TIME);
            let mut tmp = self.data[pt].load(std::sync::atomic::Ordering::SeqCst);
            let tm = i as f32 / RATE as f32;
            let v0 = bytes(tm);
            let mut m = std::cmp::max(tmp, v0);
            while let Err(t) = self.data[pt].compare_exchange_weak(
                tmp,
                m,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            ) {
                tmp = t;
                m = std::cmp::max(tmp, v0);
            }
        }
    }

    pub fn write_func(&self, mut bytes: impl FnMut(f32) -> f32, time: f32) {
        let base = *self.ptr.lock().unwrap();
        let count = (time * RATE as f32) as usize;
        for i in 0..count {
            let pt = (base + i) % (RATE * BUFFER_TIME);
            let mut tmp = self.data[pt].load(std::sync::atomic::Ordering::SeqCst);
            let tm = i as f32 / RATE as f32;
            let v0 = (bytes(tm) * 128. + 128.) as u8;
            let mut m = std::cmp::max(tmp, v0);
            while let Err(t) = self.data[pt].compare_exchange_weak(
                tmp,
                m,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            ) {
                tmp = t;
                m = std::cmp::max(tmp, v0);
            }
        }
    }
}

pub fn audio_write(bytes: &[u8]) {
    AUDIO.write(bytes);
}

pub fn audio_read(bytes: &mut [f32]) {
    AUDIO.read(bytes);
}

pub fn audio_write_byte_func(bytes: impl FnMut(f32) -> u8, time: f32) {
    AUDIO.write_byte_func(bytes, time);
}

pub fn audio_write_func(bytes: impl FnMut(f32) -> f32, time: f32) {
    AUDIO.write_func(bytes, time);
}

pub fn debug_play_sound_func(bytes: impl FnMut(f32) -> f32, time: f32) {
    audio_write_func(bytes, time);
    std::thread::sleep(Duration::from_secs_f32(time + 0.5));
}

pub fn thud_func(attack: f32, pitch: f32, fade: f32) -> (impl Fn(f32) -> f32, f32) {
    (
        move |t: f32| {
            if t < attack {
                E * (t / attack) * metallic_sound(t, pitch)
            } else {
                E * (1. - t / attack) * metallic_sound(t, pitch)
            }
        },
        attack + fade,
    )
}

pub fn metallic_sound(t: f32, pitch: f32) -> f32 {
    (t * pitch).sin() / 2.
        + (t * pitch * 2. - 0.1).sin() / 4.
        + (t * pitch * 4. + 0.5).sin() / 8.
        + (t * pitch * 8. + 1.).sin() / 16.
        + (t * pitch * 16. + 1.).sin() / 32.
}
