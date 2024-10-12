use chord::{play_all, ChordBlocked, ChordBroken, ChordBuilder, Note, Playable};
use egui::{vec2, Color32, Margin, Sense, Stroke, Vec2};
use hsv::hsv_to_rgb;
use rodio::source::{SineWave, Source};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use rustysynth::{Preset, SoundFont};
use sfont::load_soundfont;
use std::fs::read_dir;
use std::sync::Arc;
use std::time::Duration;
mod app;
mod tones;
use chord::ChordType;
use enum_iterator::all;
use tones::{
    freq_from_note, major_scale, minor_scale_har, minor_scale_mel, raise_half, A3, A4, ASH3, ASH5,
    B3, B4, BFL3, BFL4, C3, C4, C5, CSH3, CSH4, CSH5, D3, D4, E3, E4, EFL4, F3, F4, F5, G2, G4,
    GSH3, GSH4,
};
mod chord;
mod sfont;
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
