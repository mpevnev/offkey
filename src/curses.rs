use std::fmt;

use pancurses_result::Input::Character;
use pancurses_result::{initscr, Curses, Window};

use crate::note::Position;
use crate::text::Text;

pub fn init_curses() -> Result<Curses, Error> {
    let mut res = initscr().map_err(|_| "failed to initialize ncurses library")?;
    res.window_mut()
        .set_block_on_read(false)
        .map_err(|_| "failed to set input to non-blocking")?;
    Ok(res)
}

pub fn draw_state(
    curses: &mut Curses,
    text: &Text,
    position: Position,
    _freq: f64,
) -> Result<(), Error> {
    let win = curses.window_mut();
    win.erase().map_err(|_| "failed to clear the window")?;
    win.draw_box('|', '-')
        .map_err(|_| "failed to draw borders")?;
    let (maxy, maxx) = win.size().into();
    move_to(win, maxx / 2, maxy / 2)?;
    print(win, text.octave_name(position.octave))?;
    move_to(win, maxx / 2, maxy / 2 + 1)?;
    let note = &text.notes[&position.note];
    let acc = &text.accidentals[&position.accidental];
    printw(win, format_args!("{} {}", note, acc))?;
    curses.update().map_err(|_| "failed to update the screen")?;
    Ok(())
}

pub fn is_done(curses: &mut Curses) -> bool {
    let ch = curses.window_mut().read_char();
    if let Some(Character('q')) = ch {
        true
    } else {
        false
    }
}

/* ---------- error handling ---------- */

#[derive(Debug)]
pub struct Error(String);

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error(s.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

/* ---------- helpers ---------- */

fn move_to(win: &mut Window, x: i32, y: i32) -> Result<(), Error> {
    win.move_to((y, x))
        .map_err(|_| format!("failed to move the cursor to {}:{}", y, x).into())
}

fn print(win: &mut Window, text: &str) -> Result<(), Error> {
    win.put_str(text).map_err(|_| {
        format!("failed to write string '{}' to a curses window", text).into()
    })
}

fn printw(win: &mut Window, args: fmt::Arguments) -> Result<(), Error> {
    let s = format!("{}", args);
    print(win, &s)
}
