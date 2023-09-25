use anyhow::{anyhow, Result};
use crossbeam_channel::{SendError, Sender, TrySendError};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use synth::{
    waves::{Wave, Waveform},
    Note,
};

#[derive(Debug)]
pub struct MultiSender<T>(Vec<Sender<T>>);

#[allow(dead_code)]
impl<T: Copy> MultiSender<T> {
    pub fn new() -> MultiSender<T> {
        MultiSender(Vec::new())
    }

    pub fn from(senders: &[Sender<T>]) -> MultiSender<T> {
        MultiSender(Vec::from(senders))
    }

    pub fn push(&mut self, sender: Sender<T>) {
        self.0.push(sender)
    }

    pub fn try_send(&self, msg: T) -> Vec<Result<(), TrySendError<T>>> {
        self.0.iter().map(|s| s.try_send(msg)).collect()
    }

    pub fn send(&self, msg: T) -> Vec<Result<(), SendError<T>>> {
        self.0.iter().map(|s| s.send(msg)).collect()
    }
}

#[derive(Debug)]
enum ControlFlow {
    Continue,
    Quit,
}

#[derive(Debug)]
pub struct Input {
    wave: Wave,
    tx: MultiSender<Wave>,
}

impl Input {
    pub fn new(tx: MultiSender<Wave>) -> Self {
        Self {
            wave: Wave::new(Waveform::None, Note::A4, 0.3),
            tx,
        }
    }

    pub fn handle(&mut self) -> Result<()> {
        loop {
            let Ok(event) = event::read() else {
                continue;
            };
            let result = match event {
                Event::Key(key_event) => self.handle_key(key_event),
                Event::Resize(_, _) => self.send_wave(),
                _ => Ok(ControlFlow::Continue),
            }?;
            match result {
                ControlFlow::Quit => break,
                _ => (),
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key_event: KeyEvent) -> Result<ControlFlow> {
        match key_event {
            KeyEvent {
                code,
                kind: KeyEventKind::Press,
                ..
            } => match code {
                KeyCode::Char('1') => {
                    self.wave.waveform = Waveform::None;
                    self.send_wave()
                }
                KeyCode::Char('2') => {
                    self.wave.waveform = Waveform::Sin;
                    self.send_wave()
                }
                KeyCode::Char('3') => {
                    self.wave.waveform = Waveform::Saw;
                    self.send_wave()
                }
                KeyCode::Char('4') => {
                    self.wave.waveform = Waveform::Square;
                    self.send_wave()
                }
                KeyCode::Char('5') => {
                    self.wave.waveform = Waveform::Triangle;
                    self.send_wave()
                }
                KeyCode::Right => {
                    self.wave.note = self.wave.note.next_note();
                    self.send_wave()
                }
                KeyCode::Left => {
                    self.wave.note = self.wave.note.prev_note();
                    self.send_wave()
                }
                KeyCode::Up => {
                    self.wave.amp = (self.wave.amp + 0.01).clamp(0.0, 1.0);
                    self.send_wave()
                }
                KeyCode::Down => {
                    self.wave.amp = (self.wave.amp - 0.01).clamp(0.0, 1.0);
                    self.send_wave()
                }
                KeyCode::Esc => Ok(ControlFlow::Quit),
                _ => Ok(ControlFlow::Continue),
            },
            _ => Ok(ControlFlow::Continue),
        }
    }

    fn send_wave(&self) -> Result<ControlFlow> {
        self.tx
            .send(self.wave)
            .iter()
            .all(|res| !res.is_err())
            .then_some(ControlFlow::Continue)
            .ok_or(anyhow!("Error while sending wave"))
    }
}
