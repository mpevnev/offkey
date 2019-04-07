use std::collections::HashMap;
use std::io::Read;

use serde::Deserialize;

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

impl Text {
    pub fn from_reader<R: Read>(reader: R) -> serde_yaml::Result<Text> {
        serde_yaml::from_reader(reader)
    }

    /// Return an error if a required element is missing.
    ///
    /// In particular, check if all octaves from Sub Contra to the 5th are
    /// present, as well as all notes and accidentals.
    pub fn validate(&self) -> Result<(), String> {
        all_present(&self.octaves, (0..=8).map(Octave))
            .map_err(|octave| format!("There's no text for octave {}", octave.0))?;
        all_present(&self.notes, [C, D, E, F, G, A ,B].iter().cloned())
            .map_err(|note| format!("There's no text for note {:?}", note))?;
        all_present(&self.accidentals, [Natural, Sharp, Flat].iter().cloned())
            .map_err(|acc| format!("There's no text for accidental {:?}", acc))?;
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
