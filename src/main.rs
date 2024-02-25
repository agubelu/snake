mod game;
mod term;
mod snake;

pub type TermInt = u16;
pub type Coords = (u16, u16);

fn main() {
    let mut game = game::SnakeGame::new();
    game.initialize();
    game.show_intro();

    loop {
        // The main game loop takes care of exiting cleanly on CTRL+C
        game.play();
    }
}
