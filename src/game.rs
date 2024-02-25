use std::{process::exit, thread::sleep, time::Duration, cmp::max};

use crate::{Coords, TermInt};
use crate::term::TermManager;
use crate::snake::{Snake, Direction::{*, self}, MoveResult::{*, self}};

use crossterm::event::{KeyEvent, KeyModifiers, KeyCode};
use rand::seq::SliceRandom;

const TICK_INTERVAL_MS: u64 = 5;
const TICKS_UNTIL_UPDATE: u64 = 10;
const INITIAL_SNAKE_LENGTH: i16 = 6;

const SNAKE_BODY_CHAR: char = '█';
const APPLE_CHAR: char = 'O';
const DEAD_SNAKE_CHAR: char = 'X';

pub struct SnakeGame {
    width: TermInt,
    height: TermInt,
    paused: bool,
    term: TermManager,
    game_positions: Vec<Coords>,
}

impl SnakeGame {
    pub fn new() -> Self {
        SnakeGame { width: 0, height: 0, paused: false, term: TermManager::new(), game_positions: vec![] }
    }

    pub fn initialize(&mut self) {
        self.term.setup();

        let (w, h) = self.term.get_terminal_size();
        self.width = w;
        self.height = h;

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                self.game_positions.push((x, y));
            }
        }
    }

    pub fn show_intro(&mut self) {
        let lines = &[
            "Arrow keys or WASD to move",
            "Esc to pause",
            "CTRL+C to quit",
            "",
            "Press any key to begin"
        ];

        self.term.show_message(lines);

        if is_ctrl_c(&self.term.read_key_blocking()) {
            self.clean_exit()
        }

        self.term.hide_message();
    }

    pub fn play(&mut self) {
        self.term.clear();
        self.term.draw_borders(Some((self.width, self.height)));
        self.term.hide_message();

        let center = (self.width / 2, self.height / 2);
        let (max_x, max_y) = (self.width - 2, self.height - 2);

        let mut snake = Snake::new(center, INITIAL_SNAKE_LENGTH, Right);
        let mut apple = self.spawn_apple(&snake).unwrap();
        let mut dir_change: Option<Direction> = None;
        let mut ticks_until_step = TICKS_UNTIL_UPDATE;

        self.print_snake(&snake);

        loop {
            sleep(Duration::from_millis(TICK_INTERVAL_MS));

            for key_ev in self.term.read_key_events_queue() {
                match &key_ev {
                    ev if is_ctrl_c(ev) => self.clean_exit(),
                    KeyEvent { code, modifiers: _ } => match code {
                        KeyCode::Char('w') | KeyCode::Up => dir_change = Some(Up),
                        KeyCode::Char('a') | KeyCode::Left => dir_change = Some(Left),
                        KeyCode::Char('s') | KeyCode::Down => dir_change = Some(Down),
                        KeyCode::Char('d') | KeyCode::Right => dir_change = Some(Right),
                        KeyCode::Esc => self.toggle_pause(),
                        _ => {}
                    }
                }
            }

            if self.paused { continue; }

            // Not paused, count down til the next game update
            ticks_until_step -= 1;
            if ticks_until_step == 0 {
                let score = snake.body().len() as u64 - INITIAL_SNAKE_LENGTH as u64;
                ticks_until_step = if let Some(x) = TICKS_UNTIL_UPDATE.checked_sub(score / 7) {
                    max(x, 1)
                } else {
                    1
                }; // Speed up with higher scores

                if let Some(dir) = dir_change {
                    dir_change = None;
                    snake.set_direction(dir);
                }

                // Make the snake move a bit slower when going vertically, since terminal
                // characters have a higher height than width
                if matches!(snake.get_direction(), Up | Down) {
                    ticks_until_step = (ticks_until_step as f64 * 1.35).ceil() as u64;
                }

                let move_res = snake.move_step(max_x, max_y);

                match &move_res {
                    Crashed => {
                        self.game_over(&snake, score, false);
                        break;
                    },
                    Moved { new_head, old_head: _, old_tail: _ } => {
                        if *new_head == apple {
                            let opt = self.spawn_apple(&snake);
                            if opt.is_none() { // No more apples to spawn
                                self.game_over(&snake, score, true);
                                break;
                            }
                            apple = opt.unwrap();
                            snake.grow();
                        }
                        self.print_snake_update(&snake, &move_res);
                    },
                } // match
            } // Game step
        } // Game loop

        // Quit if the user CTRL+C's after the game
        if is_ctrl_c(&self.term.read_key_blocking()) {
            self.clean_exit()
        }
    }

    ///////////////////////////////////////////////////////////////////////////

    fn clean_exit(&mut self) {
        self.term.restore();
        exit(0);
    }

    fn game_over(&mut self, snake: &Snake, score: u64, win: bool) {
        let s = if win {"You won!"} else {"Game over!"};

        if !win {
            for pos in snake.body() {
                self.term.print_at(*pos, DEAD_SNAKE_CHAR);
            }
        }

        self.term.show_message(&[
            s,
            &*format!("Score: {}", score),
            "",
            "Press any key to play again,",
            "or CTRL+C to quit."
        ]);
    }

    fn spawn_apple(&mut self, snake: &Snake) -> Option<Coords> {
        let choices: Vec<&Coords> = self.game_positions.iter().filter(|pos| !snake.body().contains(pos)).collect();
        let res = choices.choose(&mut rand::thread_rng()).copied().copied();

        res.map(|apple| {
            self.term.print_at(apple, APPLE_CHAR);
            self.term.flush();
            apple
        })
    }

    fn print_snake(&mut self, snake: &Snake) {
        let snake_len = snake.body().len();

        for (i, pos) in snake.body().iter().enumerate() {
            let ch = if i == snake_len - 1 {snake.head_char()} else {SNAKE_BODY_CHAR};
            self.term.print_at(*pos, ch);
        }

        self.term.flush();
    }

    fn print_snake_update(&mut self, snake: &Snake, mov: &MoveResult) {
        if let Moved{new_head, old_head, old_tail} = mov {
            self.term.print_at(*new_head, snake.head_char());
            self.term.print_at(*old_head, SNAKE_BODY_CHAR);

            if let Some(old_tail_pos) = old_tail {
                self.term.print_at(*old_tail_pos, ' ');
            }

            self.term.flush();
        }
    }

    fn toggle_pause(&mut self) {
        if !self.paused {
            self.term.show_message(&["Paused", "Press Esc to resume", "or Ctrl+C to quit"]);
        } else {
            self.term.hide_message();
        }

        self.paused = !self.paused;
    }
}

fn is_ctrl_c(ev: &KeyEvent) -> bool {
    matches!(ev, KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL })
}
