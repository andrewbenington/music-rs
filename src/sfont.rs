use rodio::{OutputStream, OutputStreamHandle, Source};
use rustysynth::{Preset, SoundFont, Synthesizer, SynthesizerSettings};
use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

pub fn load_soundfont(file_path: &str) -> Arc<SoundFont> {
    let mut sf2 = File::open(file_path).unwrap();

    return Arc::new(SoundFont::new(&mut sf2).unwrap());
}

pub fn load_sf_source(
    soundfont: Arc<SoundFont>,
    freq: f32,
    instrument: Option<BankInstrument>,
) -> SoundFontSource {
    match instrument.clone() {
        Some(inst) => println!(
            "sf instrument: {}-{} ({})",
            inst.patch, inst.bank, inst.name
        ),
        None => println!("no instrument"),
    }

    // println!("Instruments");
    // for (i, instr) in soundfont.get_instruments().iter().enumerate() {
    //     println!("{}. {}", i, instr.get_name())
    // }
    // println!("Sample Headers");
    // for (i, sample) in sound_font.get_sample_headers().iter().enumerate() {
    //     println!("{}. {}", i, sample.get_name())
    // }
    // println!("Presets");
    // for (i, sample) in soundfont.get_presets().iter().enumerate() {
    //     println!("{}. {}", i, sample.get_name())
    // }

    let settings = SynthesizerSettings::new(44100);
    let mut synthesizer = Synthesizer::new(&soundfont, &settings).unwrap();
    // set_instrument(&mut synthesizer, 0, 10);

    synthesizer.process_midi_message(1, 0xb0, 0, instrument.clone().map_or(0, |i| i.bank));
    synthesizer.process_midi_message(
        1,
        0xc0,
        instrument.clone().map_or(3, |i: BankInstrument| i.patch),
        0,
    );
    synthesizer.note_on(1, frequency_to_midi_key(freq), 100);
    // synthesizer.note_off_all(false);

    // The output buffer (3 seconds).
    let sample_count = (3 * settings.sample_rate) as usize;
    let mut left: Vec<f32> = vec![0_f32; sample_count];
    let mut right: Vec<f32> = vec![0_f32; sample_count];

    // Render the waveform.
    synthesizer.render(&mut left[..], &mut right[..]);
    return SoundFontSource {
        synthesizer,
        samples: left,
        _index: 0,
    };
}

fn frequency_to_midi_key(frequency: f32) -> i32 {
    let midi_note = 69.0 + 12.0 * (frequency / 440.0).log2();
    midi_note.round() as i32
}

fn set_instrument(synthesizer: &mut Synthesizer, channel: i32, program: i32) {
    synthesizer.process_midi_message(channel, 0xcc, program, 0);
}

#[derive(PartialEq, Clone)]
pub struct BankInstrument {
    pub patch: i32,
    pub bank: i32,
    pub name: String,
}

impl BankInstrument {
    pub fn from_preset(preset: &Preset) -> BankInstrument {
        return BankInstrument {
            bank: preset.get_bank_number(),
            patch: preset.get_patch_number(),
            name: preset.get_name().to_string(),
        };
    }
}

// Custom Source implementation
pub struct SoundFontSource {
    synthesizer: Synthesizer,
    samples: Vec<f32>,
    _index: usize,
}

impl SoundFontSource {
    pub fn new(soundfont: Arc<SoundFont>, freq: f32, instrument: Option<BankInstrument>) -> Self {
        load_sf_source(soundfont, freq, instrument)
    }

    pub fn from_freqs(
        soundfont: Arc<SoundFont>,
        freq: f32,
        instrument: Option<BankInstrument>,
    ) -> Self {
        load_sf_source(soundfont, freq, instrument)
    }
}

impl Iterator for SoundFontSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // println!("Next: {}", self._index);
        if self._index >= self.samples.len() {
            return None;
        }
        // This is where you'd return the next sample. For now, it's just silence.
        let sample = self.samples[self._index];
        self._index += 1;

        // self.synthesizer.note_on(1, 60, 100);
        // println!("Sample: {sample}");
        return Some(sample * 10.0);
    }
}

impl Source for SoundFontSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.synthesizer.get_block_size())
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        return self.synthesizer.get_sample_rate() as u32;
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs(3))
    }
}

pub fn lower_half(note: f32, steps: i32) -> f32 {
    let mut steps_left = steps;
    let mut freq = note;
    while steps_left >= 12 {
        steps_left -= 12;
        freq *= 2.0;
    }

    while steps_left <= -12 {
        steps_left += 12;
        freq *= 0.5;
    }
    return freq * (2.0_f32).powf(steps_left as f32 / 12.0);
}
