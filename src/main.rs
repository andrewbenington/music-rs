use chord::ChordBroken;
use rodio::source::{SineWave, Source};
use rodio::{OutputStreamHandle, Sink};
use std::time::Duration;
mod app;
mod chord;
mod sfont;
mod tones;
use crate::app::MusicApp;
use eframe::egui;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Rust Music",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MusicApp>::default())
        }),
    )
}

fn play_mult_notes_repeat(
    stream_handle: &OutputStreamHandle,
    notes1: &Vec<ChordBroken>,
    notes2: &Vec<ChordBroken>,
    repeats: u64,
) {
    for _i in 0..repeats {
        play_mult_notes(&stream_handle, &notes1, &notes2);
    }
}

fn play_mult_notes(
    stream_handle: &OutputStreamHandle,
    notes1: &Vec<ChordBroken>,
    notes2: &Vec<ChordBroken>,
) {
    for note in notes1 {
        for (i, tone) in note.tones.iter().enumerate() {
            let sink = Sink::try_new(&stream_handle).unwrap();
            let note1 = SineWave::new(*tone).take_duration(Duration::from_secs_f32(note.duration));
            let note2 = SineWave::new(notes2[0].tones[i])
                .take_duration(Duration::from_secs_f32(notes2[0].duration));
            sink.append(note1.mix(note2));
            sink.sleep_until_end();
        }
    }
}

fn play_notes(stream_handle: &OutputStreamHandle, notes: &Vec<ChordBroken>) {
    for note in notes {
        for tone in note.tones.iter() {
            let sink = Sink::try_new(&stream_handle).unwrap();
            sink.append(SineWave::new(*tone).take_duration(Duration::from_secs_f32(note.duration)));
            sink.sleep_until_end();
        }
    }
}
