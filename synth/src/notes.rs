use std::mem::transmute;

use anyhow::{anyhow, Error};

const C0: f32 = 16.35160;

fn note_frequency(note: u32) -> f32 {
    C0 * f32::powf(2.0, note as f32 / 12.0)
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum Note {
    C0,
    D0b,
    D0,
    E0b,
    E0,
    F0,
    G0b,
    G0,
    A0b,
    A0,
    B0b,
    B0,
    C1,
    D1b,
    D1,
    E1b,
    E1,
    F1,
    G1b,
    G1,
    A1b,
    A1,
    B1b,
    B1,
    C2,
    D2b,
    D2,
    E2b,
    E2,
    F2,
    G2b,
    G2,
    A2b,
    A2,
    B2b,
    B2,
    C3,
    D3b,
    D3,
    E3b,
    E3,
    F3,
    G3b,
    G3,
    A3b,
    A3,
    B3b,
    B3,
    C4,
    D4b,
    D4,
    E4b,
    E4,
    F4,
    G4b,
    G4,
    A4b,
    A4,
    B4b,
    B4,
    C5,
    D5b,
    D5,
    E5b,
    E5,
    F5,
    G5b,
    G5,
    A5b,
    A5,
    B5b,
    B5,
    C6,
    D6b,
    D6,
    E6b,
    E6,
    F6,
    G6b,
    G6,
    A6b,
    A6,
    B6b,
    B6,
    C7,
    D7b,
    D7,
    E7b,
    E7,
    F7,
    G7b,
    G7,
    A7b,
    A7,
    B7b,
    B7,
    C8,
    D8b,
    D8,
    E8b,
    E8,
    F8,
    G8b,
    G8,
    A8b,
    A8,
    B8b,
    B8,
}

impl TryFrom<u32> for Note {
    type Error = Error;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let min = Self::A0 as u32;
        let max = Self::B8 as u32;
        if (min..=max).contains(&value) {
            let note = unsafe { transmute(value) };
            Ok(note)
        } else {
            Err(anyhow!("Number not a valid note"))
        }
    }
}

impl Note {
    pub fn freq(&self) -> f32 {
        note_frequency(*self as u32)
    }
    pub fn next_note(&self) -> Self {
        let min = Self::A0 as u32;
        let max = Self::B8 as u32;
        (*self as u32 + 1).clamp(min, max).try_into().unwrap()
    }
    pub fn prev_note(&self) -> Self {
        let min = Self::A0 as u32;
        let max = Self::B8 as u32;
        (*self as u32 - 1).clamp(min, max).try_into().unwrap()
    }
}
