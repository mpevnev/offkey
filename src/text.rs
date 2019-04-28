use std::collections::HashMap;
use std::io::Read;

use serde::Deserialize;
use snafu::{ResultExt, Snafu};

use crate::error;
use crate::note::{
    Accidental::{self, *},
    Note::{self, *},
    Octave,
};

#[derive(Deserialize)]
pub struct Text {
    pub octaves: HashMap<Octave, String>,
    pub notes: HashMap<Note, String>,
    pub accidentals: HashMap<Accidental, String>,
    pub low_octave: String,
    pub high_octave: String,
    pub missing_octave: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Snafu)]
#[snafu(visibility(pub))]
pub enum MissingText {
    #[snafu(display("No text for octave: {}", missing.0))]
    MissingOctave { missing: Octave },
    #[snafu(display("No text for note: {:?}", missing))]
    MissingNote { missing: Note },
    #[snafu(display("No text for accidental: {:?}", missing))]
    MissingAccidental { missing: Accidental },
}

impl Text {
    pub fn new<R: Read>(reader: R) -> Result<Self, error::Error> {
        let res: Self = serde_yaml::from_reader(reader)
            .context(error::TextDeserialization)?;
        res.validate()
            .context(error::TextValidation)?;
        Ok(res)
    }

    /// Return an error if a required element is missing.
    ///
    /// In particular, check if all octaves from Sub Contra to the 5th are
    /// present, as well as all notes and accidentals.
    fn validate(&self) -> Result<(), MissingText> {
        all_present(&self.octaves, (0..=8).map(Octave))
            .map_err(|missing| MissingText::MissingOctave { missing })?;
        all_present(&self.notes, [C, D, E, F, G, A ,B].iter().cloned())
            .map_err(|missing| MissingText::MissingNote { missing })?;
        all_present(&self.accidentals, [Natural, Sharp, Flat].iter().cloned())
            .map_err(|missing| MissingText::MissingAccidental { missing })?;
        Ok(())
    }

    pub fn octave_name(&self, octave: Octave) -> &str {
        if let Some(name) = self.octaves.get(&octave) {
            name
        } else {
            let min = self.octaves.keys().min();
            let max = self.octaves.keys().max();
            let is_low = min.map(|&min| octave < min);
            let is_high = max.map(|&max| octave < max);
            if let Some(true) = is_low {
                &self.low_octave
            } else if let Some(true) = is_high {
                &self.high_octave
            } else {
                &self.missing_octave
            }
        }
    }
}

fn all_present<T, I, V>(map: &HashMap<T, V>, mut iter: I) -> Result<(), T>
    where T: Eq + std::hash::Hash,
          I: Iterator<Item = T>,
{
    iter.try_fold((), |(), item| {
        if map.contains_key(&item) {
            Ok(())
        } else {
            Err(item)
        }
    })
}
