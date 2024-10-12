use std::{fs::read_dir, sync::Arc};

use egui::{vec2, Color32, Margin, RichText, Sense, Stroke};
use enum_iterator::all;
use hsv::hsv_to_rgb;
use rodio::{OutputStream, Sink};
use rustysynth::SoundFont;

use crate::{
    chord::{play_all, ChordBuilder, ChordType, Note},
    sfont::{load_soundfont, BankInstrument},
    tones::freq_from_note,
};

const EIGHTH: f32 = 0.4;
const SIXTEENTH: f32 = 0.2;
const THIRTYSECOND: f32 = 0.1;
pub struct MusicApp {
    name: String,
    age: u32,
    chord_builders: Vec<ChordBuilder>,
    sfonts: Vec<Arc<SoundFont>>,
    selected_chord: usize,
}

impl Default for MusicApp {
    fn default() -> Self {
        let files = read_dir("soundfonts");
        let filenames = files
            .map(|result| {
                result.map(|entry| {
                    entry.map_or("(unknown)".to_owned(), |op| {
                        format!("soundfonts/{}", op.file_name().into_string().unwrap())
                    })
                })
            })
            .unwrap();
        let default_soundfont = load_soundfont("soundfonts/The_Ultimate_Earthbound_Soundfont.sf2");
        let default_preset = &default_soundfont.get_presets()[0];
        let default_instrument = BankInstrument::from_preset(default_preset);
        let default_chord = ChordBuilder::new("A4")
            .with_sfont(default_soundfont.clone())
            .with_instrument(default_instrument.clone());

        let default_chord2 = ChordBuilder::default()
            .with_sfont(default_soundfont.clone())
            .with_instrument(default_instrument.clone());
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            chord_builders: vec![default_chord, default_chord2],
            selected_chord: 0,
            sfonts: filenames
                .map(|filename| load_soundfont(&filename))
                .collect(),
        }
    }
}

impl eframe::App for MusicApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.chord_builders.len() == 0 {
            self.chord_builders = vec![ChordBuilder::new("A4")]
        }
        // let currentChord = &self.chord_builders[currentChordIndex];
        let chord_sfont = &self.chord_builders[self.selected_chord]
            .sfont
            .clone()
            .unwrap_or(self.sfonts[0].clone());
        egui::CentralPanel::default().show(ctx, |ui| {
            // if self.note.len() >= 2 && self.note.chars().nth(1).is_some_and(|ch| ch == 'b') {
            //     self.note.replace_range(1..2, "♭")
            // }

            if self.chord_builders[self.selected_chord].sfont_name
                != chord_sfont.get_info().get_bank_name()
            {
                if let Some(new_sfont) = self.sfonts.iter().find(|sfont| {
                    return sfont.get_info().get_bank_name()
                        == self.chord_builders[self.selected_chord].sfont_name;
                }) {
                    self.chord_builders[self.selected_chord].sfont = Some(new_sfont.clone());
                    let default_preset = &new_sfont.get_presets()[0];
                    self.chord_builders[self.selected_chord].instrument =
                        Some(BankInstrument::from_preset(default_preset));
                }
            }

            ui.heading("Rust Music");
            ui.horizontal(|ui| {
                let note_label = ui.label("Enter note: ");
                ui.text_edit_singleline(&mut self.chord_builders[self.selected_chord].note)
                    .labelled_by(note_label.id);
            });

            ui.horizontal(|ui| {
                let freq_res =
                    freq_from_note(&self.chord_builders[self.selected_chord].note.as_str());
                if let Ok(freq) = freq_res {
                    if ui.button("Play Note").clicked() {
                        match &self.chord_builders[self.selected_chord].instrument {
                            Some(inst) => {
                                let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                                let sink = Sink::try_new(&stream_handle).unwrap();
                                play_all(
                                    vec![Box::new(Note {
                                        tone: freq,
                                        duration: EIGHTH * 4.0,
                                        instrument: Some(inst.clone()),
                                    })],
                                    &sink,
                                    chord_sfont.clone(),
                                );
                            }
                            None => println!("No instrument present"),
                        }
                    }
                    if ui.button("Play Chord").clicked() {
                        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                        let sink = Sink::try_new(&stream_handle).unwrap();
                        let chord_type = self.chord_builders[self.selected_chord].chord_type;
                        println!("{}", chord_sfont.get_info().get_bank_name());
                        play_all(
                            vec![Box::new(
                                self.chord_builders[self.selected_chord].blocked(EIGHTH * 4.0),
                            )],
                            &sink,
                            chord_sfont.to_owned(),
                        );
                    }

                    if ui.button("Play Arpeggio").clicked() {
                        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                        let sink = Sink::try_new(&stream_handle).unwrap();
                        let chord_type = self.chord_builders[self.selected_chord].chord_type;

                        play_all(
                            vec![Box::new(
                                self.chord_builders[self.selected_chord].arp_updown(SIXTEENTH),
                            )],
                            &sink,
                            chord_sfont.to_owned(),
                        );
                    }
                }
            });
            ui.add(
                egui::Slider::new(
                    &mut self.chord_builders[self.selected_chord].inversion,
                    -12..=12,
                )
                .text("Inversion"),
            );
            let select_label = self.chord_builders[self.selected_chord]
                .chord_type
                .to_string();

            egui::ComboBox::from_label("Chord Type")
                .selected_text(select_label)
                .show_ui(ui, |ui| {
                    for chtype in all::<ChordType>() {
                        ui.selectable_value(
                            &mut self.chord_builders[self.selected_chord].chord_type,
                            chtype,
                            chtype.to_string(),
                        );
                    }
                });

            let inst_name = self.chord_builders[self.selected_chord]
                .instrument
                .clone()
                .map_or("Select Instrument".to_owned(), |i| i.name);
            egui::ComboBox::from_label("Instrument")
                .selected_text(inst_name)
                .show_ui(ui, |ui| {
                    for (_i, instr) in chord_sfont
                        .get_presets()
                        .iter()
                        .map(|p| BankInstrument::from_preset(p))
                        .enumerate()
                    {
                        let inst_name = instr.clone().name;
                        ui.selectable_value(
                            &mut self.chord_builders[self.selected_chord].instrument,
                            Some(instr),
                            format!("{}", inst_name),
                        );
                    }
                });

            egui::ComboBox::from_label("Soundfont")
                .selected_text(self.chord_builders[self.selected_chord].sfont_name.clone())
                .show_ui(ui, |ui| {
                    for file_sfont in self.sfonts.iter() {
                        ui.selectable_value(
                            &mut self.chord_builders[self.selected_chord].sfont_name,
                            file_sfont.get_info().get_bank_name().to_string(),
                            file_sfont.get_info().get_bank_name(),
                        );
                    }
                });

            ui.horizontal_wrapped(|ui| {
                for i in 0..self.chord_builders.len() {
                    self.chord_builder(i, ui)
                }
                if ui.button("Add").clicked() {
                    let inst = BankInstrument::from_preset(&self.sfonts[0].get_presets()[0]);
                    let new_chord = ChordBuilder::default()
                        .with_sfont(self.sfonts[0].clone())
                        .with_instrument(inst.clone());
                    self.chord_builders.push(new_chord);
                }
            })
        });
    }
}

impl MusicApp {
    fn chord_builder(&mut self, cb_index: usize, ui: &mut egui::Ui) {
        let chord_builder = &self.chord_builders[cb_index];
        let (r, g, b) = rgb_from_note(&chord_builder.note);
        println!("{}: {}/{}/{}", chord_builder.note, r, g, b);
        let stroke = match self.selected_chord == cb_index {
            true => Stroke::new(2.0, Color32::WHITE),
            false => ui.visuals().widgets.noninteractive.bg_stroke,
        };
        egui::Frame::default()
            .stroke(stroke)
            .rounding(ui.visuals().widgets.noninteractive.rounding)
            .fill(Color32::from_rgb(r, g, b))
            .inner_margin(Margin::same(8.0))
            .show(ui, |ui| {
                let (rect, response) = ui.allocate_exact_size(vec2(200.0, 100.0), Sense::click());

                if response.clicked() {
                    self.selected_chord = cb_index;
                }

                let mut child_ui =
                    ui.child_ui(rect, egui::Layout::top_down(egui::Align::LEFT), None);
                child_ui.vertical(|ui| {
                    ui.horizontal_top(|ui| {
                        ui.label(
                            egui::RichText::new(chord_builder.note.clone())
                                .color(egui::Color32::WHITE)
                                .size(32.0),
                        );
                        ui.label(
                            RichText::new(chord_builder.chord_type.abbr())
                                .color(egui::Color32::WHITE)
                                .size(24.0),
                        );
                    });

                    ui.label(chord_builder.inversion.to_string());
                    ui.label(format!(
                        "{} {}-{}",
                        chord_builder
                            .instrument
                            .clone()
                            .map_or("no instrument".to_owned(), |bi| bi.name),
                        chord_builder.instrument.clone().map_or(-1, |bi| bi.bank),
                        chord_builder.instrument.clone().map_or(-1, |bi| bi.patch)
                    ));
                    ui.label(chord_builder.sfont_name.clone());
                })
            });
    }
}

fn rgb_from_note(note: &String) -> (u8, u8, u8) {
    let ch = note.chars().nth(0).unwrap_or('G').to_ascii_uppercase();
    if ch < 'A' || ch > 'G' {
        return (0, 0, 0);
    }
    let mut hue = (('G' as u8 - ch as u8) as f64) / 7.0 * 360.0;
    match note.chars().nth(1) {
        Some(fl_or_sh) => {
            if fl_or_sh == 'b' {
                if ch == 'F' || ch == 'C' {
                    hue += 1.0 / 7.0 * 360.0
                } else if ch == 'A' {
                    hue = 0.0
                } else {
                    hue += 0.5 / 7.0 * 360.0
                }
            } else if fl_or_sh == '#' {
                if ch == 'E' || ch == 'B' {
                    hue -= 1.0 / 7.0 * 360.0
                } else if ch == 'G' {
                    hue = 6.0 / 7.0 * 360.0
                } else {
                    hue -= 0.5 / 7.0 * 360.0
                }
            }
        }
        None => {}
    }

    return hsv_to_rgb(hue, 1.0, 1.0);
}
