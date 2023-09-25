use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem},
    Frame,
};
use synth::{waves::Wave, Note};

pub fn ui<B: Backend>(f: &mut Frame<B>, wave: Wave) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref())
        .split(f.size());

    let period = Note::A4.freq().recip() as f64;
    let data = wave_to_data(wave, period, 256);
    let dataset = new_dataset(
        &data,
        format!("{:?}, {:?}, {:.2?}", wave.waveform, wave.note, wave.amp),
    );
    let wave_widget = chart_wave(dataset, [0.0, 4.0 * period]);

    let controls = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints(
            [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ]
            .as_ref(),
        )
        .split(areas[1]);

    let wave_control_items = [
        ListItem::new("<1>: None"),
        ListItem::new("<2>: Sine"),
        ListItem::new("<3>: Saw"),
        ListItem::new("<4>: Square"),
        ListItem::new("<5>: Triangle"),
    ];
    let wave_controls = controls_list(&wave_control_items, "Waves");

    let note_control_items = [
        ListItem::new("<Right>: Inc. Pitch"),
        ListItem::new("<Left>: Dec. Pitch"),
    ];
    let note_controls = controls_list(&note_control_items, "Note");

    let amp_control_items = [
        ListItem::new("<Up>: Inc. Amplitude"),
        ListItem::new("<Down>: Dec. Amplitude"),
    ];
    let amp_controls = controls_list(&amp_control_items, "Amplitude");

    f.render_widget(wave_widget, areas[0]);
    f.render_widget(wave_controls, controls[0]);
    f.render_widget(note_controls, controls[1]);
    f.render_widget(amp_controls, controls[2]);
}

fn controls_list<'a>(items: &'a [ListItem], title: &'a str) -> List<'a> {
    List::new(items).block(Block::default().title(title).borders(Borders::ALL))
}

fn chart_wave(dataset: Dataset, range: [f64; 2]) -> Chart {
    Chart::new(vec![dataset])
        .block(Block::default().title("Wave").borders(Borders::ALL))
        .x_axis(Axis::default().bounds(range))
        .y_axis(
            Axis::default().bounds([-1.0, 1.0]).labels(
                ["-1.0", "0.0", "1.0"]
                    .iter()
                    .cloned()
                    .map(Span::from)
                    .collect(),
            ),
        )
}

fn wave_to_data(wave: Wave, period: f64, n_samples: u32) -> Vec<(f64, f64)> {
    let delta = 4.0 * period / (n_samples - 1) as f64;
    (0..n_samples)
        .map(|i| {
            let t = i as f64 * delta;
            (t, wave.sample(t as f32) as f64)
        })
        .collect::<Vec<_>>()
}

fn new_dataset<'a>(data: &'a [(f64, f64)], name: String) -> Dataset<'a> {
    Dataset::default()
        .name(name)
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .data(&data)
}
