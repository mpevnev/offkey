use std::fmt::Arguments;

use pancurses_result::{initscr, Curses, Window};
use pancurses_result::{Input::Character};

use crate::note::Position;
use crate::text::Text;

pub fn init_curses() -> Result<Curses, String> {
    let mut res =
        initscr().map_err(|()| String::from("Failed to initialize ncurses library"))?;
    res.window_mut()
        .set_block_on_read(false)
        .map_err(|()| String::from("Failed to set non-blocking input"))?;
    Ok(res)
}

pub fn draw_state(
    curses: &mut Curses,
    text: &Text,
    position: Position,
    _freq: f64,
) -> Result<(), String> {
    let win = curses.window_mut();
    win.erase()
        .map_err(|()| String::from("Failed to clear the window"))?;
    win.draw_box('|', '-')
        .map_err(|()| String::from("Failed to draw borders"))?;
    let (maxy, maxx) = win.size().into();
    move_to(win, maxx / 2, maxy / 2)?;
    win.put_str(text.octave_name(position.octave)).ok();
    move_to(win, maxx / 2, maxy / 2 + 1)?;
    let note = &text.notes[&position.note];
    let acc = &text.accidentals[&position.accidental];
    printw(win, format_args!("{} {}", note, acc))?;
    curses
        .update()
        .map_err(|()| String::from("Failed to update the screen"))?;
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

fn move_to(win: &mut Window, x: i32, y: i32) -> Result<(), String> {
    win.move_to((y, x))
        .map_err(|()| String::from("Failed to move the cursor"))
}

fn printw(win: &mut Window, args: Arguments) -> Result<(), String> {
    win.printw(args)
        .map_err(|()| String::from("Failed to write a string to Curses window"))
}
