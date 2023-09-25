use std::f32::consts::PI;

use crossbeam_channel::Receiver;

use crate::notes::Note;

#[derive(Debug, Clone, Copy)]
pub enum Waveform {
    None,
    Sin,
    Saw,
    Square,
    Triangle,
}

#[derive(Debug, Clone, Copy)]
pub struct Wave {
    pub waveform: Waveform,
    pub note: Note,
    pub amp: f32,
}

pub struct Oscilator {
    sample_rate: f32,
    curr_sample: f32,
    wave: Wave,
    rx: Option<Receiver<Wave>>,
}

impl Wave {
    pub fn new(waveform: Waveform, note: Note, amp: f32) -> Self {
        Self {
            waveform,
            note,
            amp,
        }
    }

    pub fn sample(&self, t: f32) -> f32 {
        match self.waveform {
            Waveform::Sin => self.sample_sin(t),
            Waveform::Saw => self.sample_saw(t),
            Waveform::Square => self.sample_square(t),
            Waveform::Triangle => self.sample_triangle(t),
            Waveform::None => 0.0,
        }
    }

    fn sample_sin(&self, t: f32) -> f32 {
        self.amp * f32::sin(self.note.freq() * 2.0 * PI * t)
    }

    fn sample_saw(&self, t: f32) -> f32 {
        let zero_one = t.mul_add(self.note.freq(), 0.5) % 1.0;
        self.amp * zero_one.mul_add(2.0, -1.0)
    }

    fn sample_square(&self, t: f32) -> f32 {
        let zero_one = t.mul_add(self.note.freq(), 0.5) % 1.0;
        self.amp * zero_one.mul_add(2.0, -1.0).signum()
    }

    fn sample_triangle(&self, t: f32) -> f32 {
        let zero_one = t.mul_add(self.note.freq(), 0.75) % 1.0;
        self.amp * zero_one.mul_add(2.0, -1.0).abs().mul_add(2.0, -1.0)
    }
}

impl Oscilator {
    pub fn new(sample_rate: u32, wave: Wave) -> Self {
        Self {
            sample_rate: sample_rate as f32,
            curr_sample: 0.0,
            wave,
            rx: None,
        }
    }

    fn inc_sample(&mut self) {
        self.curr_sample += 1.0;
        self.curr_sample %= self.wave.note.freq().recip() * self.sample_rate;
    }

    pub fn sample(&mut self) -> f32 {
        match &self.rx {
            Some(rx) => {
                if let Ok(wave) = rx.try_recv() {
                    self.wave = wave;
                }
            }
            _ => (),
        }
        self.inc_sample();
        self.wave.sample(self.curr_sample / self.sample_rate)
    }

    pub fn add_receiver(&mut self, rx: Receiver<Wave>) {
        self.rx = Some(rx);
    }
}
