#![deny(bare_trait_objects)]
#![warn(clippy::all)]

mod analyser;
mod cli;
mod curses;
mod error;
mod input;
mod mic;
mod note;
mod sample;
mod text;

use std::fs::File;

use analyser::Analyser;
use cli::CLIData;
use curses::{draw_state, init_curses, is_done};
use mic::{open_microphone, MicSettings};
use note::Position;
use text::Text;

use snafu::ResultExt;

fn main() -> Result<(), error::Error> {
    // Do note that this one will kill the program in case of errors.
    let cli = CLIData::new();
    let set = MicSettings {
        access: Some(alsa::pcm::Access::RWInterleaved),
        ..MicSettings::default()
    };
    let mic = open_microphone(&cli.device_name, set).context(error::AlsaDeviceSetup)?;
    let mut analyser =
        Analyser::<'_, f64>::new(&mic, 1750).context(error::AnalyserSetup)?;
    let strings_file = File::open(&cli.text_data_file).context(error::TextFileRead)?;
    let text = Text::new(strings_file)?;
    let mut curses = init_curses().context(error::Curses)?;
    eprintln!("ENTERED THE LOOP");
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if let Err(error) = analyser.read_data() {
            analyser.recover(error).context(error::AlsaProcessing)?;
        } else {
            analyser.do_fft();
        }
        if let Some(dominant) = analyser.dominant_frequency() {
            let pos = Position::from_frequency(dominant);
            if let Some(pos) = pos {
                draw_state(&mut curses, &text, pos, dominant)
                    .context(error::Curses)?;
            }
            if is_done(&mut curses) {
                break;
            }
        }
    }
    eprintln!("DONE");
    Ok(())
}

/*
fn read_frames<'a, T>(
    mic: &PCM,
    input: &mut Input<'a, T>,
    output: &mut [Complex<T>],
) -> Result<(), String>
where
    T: FromAnySample + Num + Clone,
{
    if let Err(e) = input.read() {
        match e.errno() {
            Some(Errno::EAGAIN) => {}
            Some(Errno::EPIPE) => {
                eprintln!("{}", e);
                mic.try_recover(e, true)
                    .map_err(|e| format!("Failed to recover from a EPIPE error: {}", e))?
            },
            Some(_) => return Err(format!("Failed to read input: {}", e)),
            None => {}
        }
    };
    input.copy_frames(output);
    Ok(())
}
*/
