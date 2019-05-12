use std::cmp::Ordering;
use std::ops::Sub;

use rustfft::num_traits::Float;
use serde::Deserialize;

use Accidental::*;
use Note::*;

/* ---------- Constants ---------- */

// So sad that `const fn` is not yet stable.

pub const SEMITONES_PER_OCTAVE: i32 = 12;
pub const SEMITONES_PER_OCTAVE_F: f64 = 12.0;
pub const SUB_CONTRA: Octave = Octave(0);
pub const SUB_CONTRA_A_FREQ: f64 = 27.5;

/* ---------- types ---------- */

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub octave: Octave,
    pub note: Note,
    pub accidental: Accidental,
}

/// An octave
///
/// This is a very thin wrapper over an octave number according to the
/// scientific naming system. Sub Contra octave is 0, and the First octave is
/// 4.
#[serde(transparent)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Hash, Deserialize)]
pub struct Octave(pub i32);

#[repr(i32)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Hash, Deserialize)]
pub enum Note {
    C = 0,
    D = 2,
    E = 4,
    F = 5,
    G = 7,
    A = 9,
    B = 11,
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Deserialize)]
pub enum Accidental {
    Flat,
    Sharp,
    Natural,
}

/* ---------- joint manipulation ---------- */

impl Position {
    /// Position from a frequency.
    pub fn from_frequency<T: Float>(freq: T) -> Option<Self> {
        let freq = freq.to_f64()
            .and_then(|f| if f > 0.0 { Some(f) } else { None })?;
        let octave = Octave::from_frequency(freq);
        let offset = octave.frequency_offset(freq);
        if let Some(note) = Note::from_semitone(offset) {
            Some(Position::from_parts(octave, note, Natural))
        } else {
            // Need to figure out if a flat or a sharp is more appropriate.
            let lower = Note::from_semitone(offset - 1)
                .and_then(|note| if note.can_be_sharp() { Some(note) } else { None })
                .map(|note| (note, Sharp));
            let upper = Note::from_semitone(offset + 1)
                .and_then(|note| if note.can_be_flat() { Some(note) } else { None })
                .map(|note| (note, Flat));
            let note_and_acc = lower.or(upper);
            note_and_acc.map(|(note, acc)| Position::from_parts(octave, note, acc))
        }
    }

    /// Position from a combination of an octave, note and accidental.
    pub fn from_parts(octave: Octave, note: Note, accidental: Accidental) -> Self {
        Position {
            octave,
            note,
            accidental,
        }
    }

    /// Semitone offset of this position wrt C in the Sub Contra octave.
    pub fn semitone_offset(&self) -> i32 {
        let from_octave = self.octave.0 * SEMITONES_PER_OCTAVE;
        let from_note: i32 = self.note.semitone_offset();
        let from_accidental: i32 = self.accidental.semitone_shift();
        from_octave + from_note + from_accidental
    }

}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.semitone_offset() == other.semitone_offset()
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.semitone_offset().partial_cmp(&other.semitone_offset())
    }
}

/* ---------- octave manipulation ---------- */

impl Octave {
    pub fn from_frequency(freq: f64) -> Self {
        let octave = (freq / SUB_CONTRA.lower_frequency()).log2();
        Octave(octave.floor() as i32)
    }

    pub fn lower_frequency(self) -> f64 {
        self.note_frequency(C, Natural)
    }

    pub fn frequency_offset(self, freq: f64) -> i32 {
        let semitone = (freq / self.lower_frequency()).log2() * SEMITONES_PER_OCTAVE_F;
        semitone.round() as i32
    }

    pub fn note_frequency(self, note: Note, acc: Accidental) -> f64 {
        let semitones_from_octave = self.0 * SEMITONES_PER_OCTAVE;
        let semitones_from_notes = note - A;
        let total = f64::from(
            semitones_from_octave + semitones_from_notes + acc.semitone_shift(),
        );
        SUB_CONTRA_A_FREQ * (total / SEMITONES_PER_OCTAVE_F).exp2()
    }
}

impl Eq for Octave { }

impl Ord for Octave {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

/* ---------- note manipulation ---------- */

impl Note {
    /// Semitone offset of this note from C in the same octave.
    pub fn semitone_offset(self) -> i32 {
        self as i32
    }

    pub fn from_semitone(semitone: i32) -> Option<Self> {
        [C, D, E, F, G, A, B].iter()
            .find(|note| note.semitone_offset() == semitone)
            .cloned()
    }

    pub fn can_be_flat(self) -> bool {
        match self {
            C => false,
            D => true,
            E => true,
            F => false,
            G => true,
            A => true,
            B => true,
        }
    }

    pub fn can_be_sharp(self) -> bool {
        match self {
            C => true,
            D => true,
            E => false,
            F => true,
            G => true,
            A => true,
            B => false,
        }
    }
}

impl Sub for Note {
    type Output = i32;

    fn sub(self, other: Self) -> Self::Output {
        self.semitone_offset() - other.semitone_offset()
    }
}

impl Eq for Note { }

/* ---------- accidental manipulation ---------- */

impl Accidental {
    /// The shift in semitones this accidental represents.
    pub fn semitone_shift(self) -> i32 {
        match self {
            Flat => -1,
            Sharp => 1,
            Natural => 0,
        }
    }
}

impl Default for Accidental {
    fn default() -> Self {
        Natural
    }
}

impl Eq for Accidental { }
