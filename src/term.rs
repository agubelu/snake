use crate::{TermInt, Coords};
use std::{io::{Stdout, Write, stdout}, time::Duration};

use crossterm::{cursor, execute, queue, style, terminal};
use crossterm::terminal::{ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::event::{Event, KeyEvent, read, poll};

pub struct TermManager {
    width: TermInt,
    height: TermInt,
    stdout: Stdout,
    screen: Vec<char>,
    current_msg: Option<Message>,
}

struct Message {
    top_left: Coords,
    width: TermInt,
    height: TermInt,
}

impl TermManager {
    pub fn new() -> Self {
        let (width, height) = terminal::size().expect("Error reading size.");
        let stdout = stdout();
        let screen = vec![' '; width as usize * height as usize];
        TermManager { width, height, stdout, screen, current_msg: None }
    }

    pub fn setup(&mut self) {
        execute!(self.stdout, EnterAlternateScreen).expect("Error entering alt screen");
        self.set_raw_mode(true);
        self.set_cursor_visibility(false);
        self.set_cursor_blink(false);
    }

    pub fn restore(&mut self) {
        self.set_raw_mode(false);
        self.set_cursor_visibility(true);
        self.set_cursor_blink(true);
        execute!(self.stdout, LeaveAlternateScreen).expect("Error leaving alt screen");
    }

    pub fn read_key_blocking(&self) -> KeyEvent {
        loop {
            if let Event::Key(ev) = read().unwrap() {
                return ev;
            }
        }
    }

    pub fn read_key_events_queue(&self) -> Vec<KeyEvent> {
        let mut events = vec![];

        while poll(Duration::from_millis(1)).unwrap() {
            if let Event::Key(ev) = read().unwrap() {
                events.push(ev);
            }
        }

        events
    }

    pub fn get_terminal_size(&self) -> Coords {
        (self.width, self.height)
    }

    pub fn draw_borders(&mut self, size: Option<Coords>) {
        let (width, height) = match size {
            Some((x, y)) => (x, y),
            None => (self.width, self.height)
        };

        let end_x = width - 1;
        let end_y = height - 1;

        for x in 0..width {
            let ch = if x == 0 || x == width - 1 {'+'} else {'-'};
            self.print_at((x, 0), ch);
            self.print_at((x, end_y), ch);
        }

        for y in 1..height - 1 {
            self.print_at((0, y), '|');
            self.print_at((end_x, y), '|');
        }

        self.flush();
    }

    pub fn show_message(&mut self, lines: &[&str]) {
        if self.has_message() {
            self.hide_message();
        }

        let msg_height = (lines.len() + 2) as TermInt;
        let msg_width = (lines.iter().map(|x| x.len()).max().unwrap() + 2) as TermInt;
        let center = (self.width / 2, self.height / 2);
        let top_left = (center.0 - msg_width as TermInt / 2, center.1 - msg_height as TermInt / 2);

        // Print the top and bottom empty lines
        for y in [top_left.1, top_left.1 + msg_height - 1].iter() {
            for x_diff in 0..msg_width {
                self.print_at_no_save((top_left.0 + x_diff, *y), ' ');
            }
        }

        // Print the message lines
        for (i, line) in lines.iter().enumerate() {
            let padded_line = format!("{line: ^width$}", line = line, width = msg_width as usize);
            let y = top_left.1 + i as TermInt + 1;
            for (x_diff, ch) in padded_line.char_indices() {
                self.print_at_no_save((top_left.0 + x_diff as TermInt, y), ch);
            }
        }

        self.current_msg = Some(Message::new(msg_width, msg_height, top_left));
        self.flush();
    }

    pub fn hide_message(&mut self) {
        if !self.has_message() {
            return;
        }

        let msg = self.current_msg.take().unwrap(); // take() sets current_msg to None
        let top_left = msg.top_left();

        // Restore the content from the screen buffer
        for y_diff in 0..msg.height() {
            for x_diff in 0..msg.width() {
                let (x, y) = (top_left.0 + x_diff, top_left.1 + y_diff);
                let ch = self.screen[self.width as usize * y as usize + x as usize];
                self.print_at_no_save((x, y), ch);
            }
        }

        self.flush();
    }

    pub fn print_at(&mut self, pos: Coords, ch: char) {
        queue!(self.stdout, cursor::MoveTo(pos.0, pos.1), style::Print(ch)).unwrap();
        self.screen[self.width as usize * pos.1 as usize + pos.0 as usize] = ch;
    }

    pub fn clear(&mut self) {
        execute!(self.stdout, terminal::Clear(ClearType::All)).expect("Error clearing.");
        self.screen = vec![' '; self.width as usize * self.height as usize]
    }

    pub fn flush(&mut self) {
        self.stdout.flush().expect("Error flushing.");
    }

    pub fn has_message(&self) -> bool {
        self.current_msg.is_some()
    }

    ///////////////////////////////////////////////////////////////////////////

    fn print_at_no_save(&mut self, pos: Coords, ch: char) {
        // To be used for printing messages, where we don't wanna overwrite our
        // local buffer to restore it when the message is hidden
        queue!(self.stdout, cursor::MoveTo(pos.0, pos.1), style::Print(ch)).unwrap();
    }

    fn set_raw_mode(&self, option: bool) {
        let res = if option {
            terminal::enable_raw_mode()
        } else {
            terminal::disable_raw_mode()
        };

        res.expect("Error setting raw mode.");
    }

    fn set_cursor_blink(&mut self, option: bool) {
        let res = if option {
            execute!(self.stdout, cursor::EnableBlinking)
        } else {
            execute!(self.stdout, cursor::DisableBlinking)
        };

        res.expect("Error setting cursor blink.");
    }

    fn set_cursor_visibility(&mut self, option: bool) {
        let res = if option {
            execute!(self.stdout, cursor::Show)
        } else {
            execute!(self.stdout, cursor::Hide)
        };

        res.expect("Error setting cursor visibility.");
    }
}

impl Message {
    pub fn new(width: TermInt, height: TermInt, top_left: Coords) -> Self {
        Message { width, height, top_left }
    }

    pub fn width(&self) -> TermInt {
        self.width
    }

    pub fn height(&self) -> TermInt {
        self.height
    }

    pub fn top_left(&self) -> Coords {
        self.top_left
    }
}
