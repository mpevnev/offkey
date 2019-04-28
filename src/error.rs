use std::io;

use snafu::Snafu;

use crate::curses;
use crate::text;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("ALSA error while setting up the input device: {}", source))]
    AlsaDeviceSetup { source: alsa::Error },
    #[snafu(display("ALSA error while creaing an analyser: {}", source))]
    AnalyserSetup { source: alsa::Error },
    #[snafu(display("ALSA error while processing: {}", source))]
    AlsaProcessing { source: alsa::Error },
    #[snafu(display("Failed to read text data: {}", source))]
    TextFileRead { source: io::Error },
    #[snafu(display("Failed to deserialize text data: {}", source))]
    TextDeserialization { source: serde_yaml::Error },
    #[snafu(display("Invalid text data: {}", source))]
    TextValidation { source: text::MissingText },
    #[snafu(display("Curses error: {}", source))]
    Curses { source: curses::Error },
}
