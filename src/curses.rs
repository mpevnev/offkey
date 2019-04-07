use std::fmt::Arguments;

use pancurses_result::{initscr, Curses, Window};
use pancurses_result::{Input::Character};

use crate::note::Position;

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
    position: Position,
    _freq: f64,
) -> Result<(), String> {
    let win = curses.window_mut();
    win.erase()
        .map_err(|()| String::from("Failed to clear the window"))?;
    win.draw_box('|', '-')
        .map_err(|()| String::from("Failed to draw borders"))?;
    let (maxx, maxy) = win.size().into();
    move_to(win, maxx / 2, maxy / 2)?;
    printw(win, format_args!("Octave: {:?}", position.octave))?;
    move_to(win, maxx / 2, maxy / 2 + 1)?;
    printw(
        win,
        format_args!("{:?} {:?}", position.note, position.accidental),
    )?;
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
