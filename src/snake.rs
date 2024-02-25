use crate::{Coords, TermInt};
use Direction::*;
use MoveResult::*;

#[derive(Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}

pub enum MoveResult {
    Moved { new_head: Coords, old_head: Coords, old_tail: Option<Coords> },
    Crashed
}

pub struct Snake {
    body: Vec<Coords>,
    direction: Direction,
    grow_next_move: bool,
}

impl Snake {
    pub fn new(pos: Coords, size: i16, direction: Direction) -> Self {
        let diff = match &direction {
            Up => (0, -1),
            Down => (0, 1),
            Left => (-1, 0),
            Right => (1, 0),
        };

        let body = (0..size).rev()
            .map(|i| (pos.0 as i16 - diff.0 * i, pos.1 as i16 - diff.1 * i))
            .map(|(x, y)| (x as TermInt, y as TermInt))
            .collect();
        Snake { body, direction, grow_next_move: false }
    }

    pub fn body(&self) -> &[Coords] {
        &self.body
    }

    pub fn move_step(&mut self, max_x: TermInt, max_y: TermInt) -> MoveResult {
        let old_head = *self.body.last().unwrap();

        let new_head = match &self.direction {
            Up => (old_head.0, old_head.1 - 1),
            Down => (old_head.0, old_head.1 + 1),
            Left => (old_head.0 - 1, old_head.1),
            Right => (old_head.0 + 1, old_head.1),
        };

        if new_head.0 == 0 || new_head.1 == 0 || new_head.0 > max_x || 
           new_head.1 > max_y || self.body()[1..].contains(&new_head) {
               return Crashed;
           }

        self.body.push(new_head);

        if self.grow_next_move {
            self.grow_next_move = false;
            Moved { new_head, old_head, old_tail: None }
        } else {
            let old_tail = self.body.drain(0..1).next().unwrap();
            Moved { new_head, old_head, old_tail: Some(old_tail) }
        }
    }

    pub fn set_direction(&mut self, new_direction: Direction) {
        match (&new_direction, &self.direction) {
            (Up, Down) | (Down, Up) | (Right, Left) | (Left, Right) => {},
            _ => self.direction = new_direction,
        };
    }

    pub fn get_direction(&self) -> Direction {
        self.direction
    }

    pub fn grow(&mut self) {
        self.grow_next_move = true;
    }

    pub fn head_char(&self) -> char {
        match self.direction {
            Up => '^',
            Down => 'v',
            Left => '<',
            Right => '>',
        }
    }
}
