use crate::sfont::BankInstrument;
use enum_iterator::{all, cardinality, Sequence};
use rand;
use rodio::{source::TakeDuration, Sink, Source};
use rustysynth::SoundFont;
use std::{collections::VecDeque, fmt::Display, sync::Arc, time::Duration};

use crate::{
    sfont::{load_sf_source, SoundFontSource},
    tones::{freq_from_note, maj_3rd_up, min_3rd_up, A4},
};
const SAMPLE_RATE: u32 = 44100;
const VOLUME: f32 = 0.2;

pub struct Note {
    pub duration: f32,
    pub tone: f32,
    pub instrument: Option<BankInstrument>,
}

pub struct Rest {
    pub duration: f32,
}

pub struct ChordBroken {
    pub duration: f32,
    pub tones: Vec<f32>,
    pub instrument: Option<BankInstrument>,
}
pub struct ChordBlocked {
    pub duration: f32,
    pub tones: Vec<f32>,
    pub instrument: Option<BankInstrument>,
}

impl Playable for Note {
    fn add_to_sink(&self, sink: &Sink, soundfont: Arc<SoundFont>) {
        sink.append(
            SoundFontSource::new(soundfont, self.tone, self.instrument.clone())
                .take_duration(Duration::from_secs_f32(self.duration)),
        );
    }
}

impl Playable for Rest {
    fn add_to_sink(&self, sink: &Sink, soundfont: Arc<SoundFont>) {
        sink.append(
            load_sf_source(soundfont, 0.0, None)
                .take_duration(Duration::from_secs_f32(self.duration)),
        );
    }
}
pub trait Playable {
    fn add_to_sink(&self, sink: &Sink, soundfont: Arc<SoundFont>);
}

impl Playable for ChordBroken {
    fn add_to_sink(&self, sink: &Sink, soundfont: Arc<SoundFont>) {
        match self.instrument.clone() {
            Some(inst) => println!(
                "chord instrument: {}-{} ({})",
                inst.patch, inst.bank, inst.name
            ),
            None => println!("no instrument"),
        }

        for freq in &self.tones {
            let this_source =
                SoundFontSource::new(soundfont.clone(), *freq, self.instrument.clone());
            sink.append(this_source.take_duration(Duration::from_secs_f32(self.duration)));
        }
    }
}

pub fn brk_chrd_box(tones: Vec<f32>, duration: f32) -> Box<ChordBroken> {
    return Box::new(ChordBroken {
        duration,
        tones,
        instrument: None,
    });
}
pub fn note_box(tone: f32, duration: f32) -> Box<Note> {
    return Box::new(Note {
        duration,
        tone,
        instrument: None,
    });
}

fn soundfont_source(
    soundfont: Arc<SoundFont>,
    freq: f32,
    instrument: Option<BankInstrument>,
    duration: f32,
) -> TakeDuration<SoundFontSource> {
    return SoundFontSource::new(soundfont, freq, instrument)
        .take_duration(Duration::from_secs_f32(duration));
}

impl Playable for ChordBlocked {
    fn add_to_sink(&self, sink: &Sink, soundfont: Arc<SoundFont>) {
        let mut mixed_source: Option<Box<dyn Source<Item = f32> + Send>> = None;
        for freq in &self.tones {
            // let sine = SineWave::new(*freq)
            //     .take_duration(Duration::from_secs_f32(self.duration))
            //     .amplify(VOLUME);

            let sfont = load_sf_source(soundfont.clone(), *freq, self.instrument.clone())
                .take_duration(Duration::from_secs_f32(self.duration));
            mixed_source = match mixed_source {
                Some(prev_mix) => Some(Box::new(prev_mix.mix(sfont))),
                None => Some(Box::new(sfont)),
            };
        }

        if let Some(source) = mixed_source {
            sink.append(source);
        } else {
            eprintln!("No tones to play.");
        }
    }
}

pub fn play_all(playables: Vec<Box<dyn Playable>>, sink: &Sink, soundfont: Arc<SoundFont>) {
    for playable in playables {
        playable.add_to_sink(sink, soundfont.clone())
    }
    sink.sleep_until_end()
}

#[derive(Clone, Copy, PartialEq, Sequence)]
pub enum ChordType {
    MajTri,
    MinTri,
    DimTri,
    Dom7th,
    Min7th,
    Maj7th,
    Aug5th,
    Aug7th,
}

impl ChordType {
    fn index(i: usize) -> ChordType {
        return all::<ChordType>().collect::<Vec<_>>()[i];
    }

    pub fn abbr(self) -> String {
        match self {
            ChordType::MajTri => "Δ".to_owned(),
            ChordType::MinTri => "min".to_owned(),
            ChordType::DimTri => "o".to_owned(),
            ChordType::Dom7th => "7".to_owned(),
            ChordType::Min7th => "min7".to_owned(),
            ChordType::Maj7th => "maj7".to_owned(),
            ChordType::Aug5th => "+".to_owned(),
            ChordType::Aug7th => "+7".to_owned(),
        }
    }
}

impl Display for ChordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ChordType::MajTri => write!(f, "Major Triad"),
            ChordType::MinTri => write!(f, "Minor Triad"),
            ChordType::DimTri => write!(f, "Diminished Triad"),
            ChordType::Dom7th => write!(f, "Dominant 7th"),
            ChordType::Min7th => write!(f, "Minor 7th"),
            ChordType::Maj7th => write!(f, "Major 7th"),
            ChordType::Aug5th => write!(f, "Augmented 5th"),
            ChordType::Aug7th => write!(f, "Augmented 7th"),
        }
    }
}

pub struct ChordBuilder {
    tones: Vec<f32>,
    pub chord_type: ChordType,
    pub note: String,
    pub sfont: Option<Arc<SoundFont>>,
    pub sfont_name: String,
    pub instrument: Option<BankInstrument>,
    pub inversion: i32,
}

fn apply_inversion(tones: Vec<f32>, inversions: i32) -> Vec<f32> {
    if tones.len() == 0 {
        return tones;
    }
    let mut remaining = inversions;
    let mut deque = VecDeque::from(tones.clone());
    while remaining > 0 {
        remaining -= 1;
        let first = deque.pop_front().unwrap();
        deque.push_back(first * 2.0)
    }
    while remaining < 0 {
        remaining += 1;
        let last = deque.pop_back().unwrap();
        deque.push_front(last * 0.5)
    }
    return Vec::from(deque);
}

impl ChordBuilder {
    pub fn new(note: &str) -> ChordBuilder {
        let freq = match freq_from_note(&note) {
            Ok(f) => f,
            Err(_) => A4,
        };
        return ChordBuilder {
            tones: vec![],
            chord_type: ChordType::MajTri,
            note: note.to_string(),
            sfont: None,
            sfont_name: "".to_owned(),
            instrument: None,
            inversion: 0,
        };
    }

    fn get_freq(&self) -> f32 {
        return match freq_from_note(&self.note) {
            Ok(f) => f,
            Err(_) => A4,
        };
    }

    pub fn with_type(mut self, chord_type: ChordType) -> Self {
        self.chord_type = chord_type;
        return self;
    }

    pub fn with_sfont(mut self, sfont: Arc<SoundFont>) -> Self {
        self.sfont_name = sfont.get_info().get_bank_name().to_string();
        self.sfont = Some(sfont);
        return self;
    }

    pub fn default() -> ChordBuilder {
        let note_index = (rand::random::<f32>() * 7.0) as u32;
        let ch = ('A' as u32) + note_index;
        let mut note = char::from_u32(ch).unwrap_or('A').to_string();

        let r_branch = rand::random::<f32>();
        if r_branch < 0.33 {
            if ch != 'F' as u32 && ch != 'C' as u32 {
                note += "b";
            }
        } else if r_branch < 0.67 {
            if ch != 'E' as u32 && ch != 'B' as u32 {
                note += "#";
            }
        }
        let octave = (rand::random::<f32>() * 3.0) as u32 + 2;
        note.push_str(octave.to_string().as_str());

        let ct_index = (rand::random::<f32>() * (cardinality::<ChordType>() as f32)) as usize;

        return ChordBuilder {
            note,
            tones: vec![],
            chord_type: ChordType::index(ct_index),
            sfont: None,
            sfont_name: "".to_owned(),
            instrument: None,
            inversion: 0,
        };
    }

    pub fn with_instrument(mut self, instrument: BankInstrument) -> Self {
        self.instrument = Some(instrument);
        return self;
    }

    pub fn get_tones(&self) -> Vec<f32> {
        let tones = get_tones(self.get_freq(), &self.chord_type);
        if self.inversion != 0 {
            return apply_inversion(tones, self.inversion);
        }
        return tones;
    }

    pub fn blocked(&self, duration: f32) -> ChordBlocked {
        return ChordBlocked {
            duration: duration,
            tones: self.get_tones(),
            instrument: self.instrument.clone(),
        };
    }

    pub fn arp_up(&self, duration: f32) -> ChordBroken {
        return ChordBroken {
            duration,
            tones: self.get_tones(),
            instrument: self.instrument.clone(),
        };
    }

    pub fn arp_down(&self, duration: f32) -> ChordBroken {
        let mut tones = self.get_tones();
        tones.reverse();

        return ChordBroken {
            duration,
            tones,
            instrument: self.instrument.clone(),
        };
    }

    pub fn arp_updown(&self, duration: f32) -> ChordBroken {
        println!("arp updown");
        let mut tones = self.get_tones();
        println!("Length: {}", tones.len());
        tones.push(*tones.get(0).expect("Invalid index on tone vector") * 2.0);
        for i in (0..tones.len() - 1).rev() {
            println!(
                "{}. {}",
                i,
                *tones.get(i).expect("Invalid index on tone vector")
            );
            tones.push(*tones.get(i).expect("Invalid index on tone vector"));
        }

        return ChordBroken {
            duration,
            tones,
            instrument: self.instrument.clone(),
        };
    }

    pub fn to_string(&self) -> String {
        return self.note.clone() + self.chord_type.abbr().as_str();
    }
}

fn get_tones(root: f32, chord_type: &ChordType) -> Vec<f32> {
    return match chord_type {
        ChordType::MajTri => build_maj_tri(root),
        ChordType::MinTri => build_min_tri(root),
        ChordType::DimTri => build_dim_tri(root),
        ChordType::Dom7th => build_dom_7th(root),
        ChordType::Maj7th => build_maj_7th(root),
        ChordType::Min7th => build_min_7th(root),
        ChordType::Aug5th => build_aug_5th(root),
        ChordType::Aug7th => build_aug_7th(root),
    };
}

fn build_maj_tri(root: f32) -> Vec<f32> {
    let third = maj_3rd_up(root);
    let fifth = min_3rd_up(third);
    return vec![root, third, fifth];
}

fn build_min_tri(root: f32) -> Vec<f32> {
    let third = min_3rd_up(root);
    let fifth = maj_3rd_up(third);
    return vec![root, third, fifth];
}

fn build_dim_tri(root: f32) -> Vec<f32> {
    let third = min_3rd_up(root);
    let fifth = min_3rd_up(third);
    return vec![root, third, fifth];
}

fn build_dom_7th(root: f32) -> Vec<f32> {
    let third = maj_3rd_up(root);
    let fifth = min_3rd_up(third);
    let seventh = min_3rd_up(fifth);
    return vec![root, third, fifth, seventh];
}

fn build_min_7th(root: f32) -> Vec<f32> {
    let third = min_3rd_up(root);
    let fifth = maj_3rd_up(third);
    let seventh = min_3rd_up(fifth);
    return vec![root, third, fifth, seventh];
}

fn build_maj_7th(root: f32) -> Vec<f32> {
    let third = maj_3rd_up(root);
    let fifth = min_3rd_up(third);
    let seventh = maj_3rd_up(fifth);
    return vec![root, third, fifth, seventh];
}

fn build_aug_5th(root: f32) -> Vec<f32> {
    let third = maj_3rd_up(root);
    let fifth = maj_3rd_up(third);
    return vec![root, third, fifth];
}

fn build_aug_7th(root: f32) -> Vec<f32> {
    let third = maj_3rd_up(root);
    let fifth = maj_3rd_up(third);
    let seventh = min_3rd_up(fifth);
    return vec![root, third, fifth, seventh];
}
