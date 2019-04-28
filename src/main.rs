#![warn(clippy::all)]

mod analyser;
mod curses;
mod input;
mod mic;
mod note;
mod sample;
mod text;

use std::env::args;
use std::fs::File;

use analyser::Analyser;
use curses::{draw_state, init_curses, is_done};
use mic::{open_microphone, MicSettings};
use note::Position;
use text::Text;

fn main() -> Result<(), String> {
    let device_name = args().nth(1).ok_or("usage: offkey <input device> <strings file>")?;
    let strings_file = args().nth(2).ok_or("usage: offkey <input device> <strings file>")?;
    let set = MicSettings {
        access: Some(alsa::pcm::Access::RWInterleaved),
        ..MicSettings::default()
    };
    let mic = open_microphone(&device_name, set)
        .map_err(|e| format!("Failed to open the microphone: {}", e))?;
    let mut analyser = Analyser::<'_, f64>::new(&mic, 1750)
        .map_err(|e| format!("Failed to initialize the analyser: {}", e))?;
    let strings_file = File::open(strings_file)
        .map_err(|e| format!("Failed to open strings file: {}", e))?;
    let text = Text::from_reader(strings_file)
        .map_err(|e| format!("Failed to deserialize strings: {}", e))?;
    text.validate()?;
    let mut curses = init_curses()?;
    eprintln!("ENTERED THE LOOP");
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        analyser.read_data().ok();
        analyser.do_fft();
        if let Some(dominant) = analyser.dominant_frequency() {
            let pos = Position::from_frequency(dominant);
            if let Some(pos) = pos {
                draw_state(&mut curses, &text, pos, dominant)?;
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
