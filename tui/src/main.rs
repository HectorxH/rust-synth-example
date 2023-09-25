mod input;
mod terminal;
mod ui;

use anyhow::Result;
use input::{Input, MultiSender};
use synth::{
    waves::{Wave, Waveform},
    AudioDevice, Note, StreamTrait, Synth,
};
use terminal::{restore_terminal, setup_terminal};
use ui::ui;

fn main() -> Result<()> {
    let device = AudioDevice::default()?;
    let mut synth = Synth::new(device)?
        .channels(2)?
        .buffer_size(512)?
        .sample_rate(44100)?;

    let wave = Wave::new(Waveform::None, Note::A4, 0.3);

    let (s_stream, r_stream) = crossbeam_channel::unbounded();
    let (s_main, r_main) = crossbeam_channel::unbounded();
    let multi_s = MultiSender::from(&[s_stream, s_main]);
    let stream = synth.new_output_stream_chan::<f32>(wave, r_stream.clone())?;
    stream.play()?;

    let mut terminal = setup_terminal()?;
    terminal.draw(|frame| ui(frame, wave))?;

    std::thread::spawn(move || Input::new(multi_s).handle());

    while let Ok(wave) = r_main.recv() {
        terminal.draw(|frame| ui(frame, wave))?;
    }

    restore_terminal(&mut terminal)?;

    Ok(())
}
