use anyhow::Result;
use crate::board::Board;

pub struct Bot {
    board: Board,
}

impl Bot {
    pub fn new() -> Self {
        Self {
            board: Board::starting_position()
        }
    }

    pub fn start_uci(&mut self) -> Result<()> {
        loop {
            let input = self.get_uci_move_input();
            let split_input: Vec<&str> = input.split_whitespace().collect();
            let input_slice = split_input.as_slice();
            match input_slice {
                ["uci"] => println!("uciok"),
                ["ucinewgame"] => {},
                ["isready"] => println!("readyok"),
                ["position", ..] => self.handle_position_command(input_slice),
                _ => continue,
            }
        }
    }

    fn get_uci_move_input(&self) -> String {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("failed to read line");

        input.trim().to_owned()
    }

    fn handle_position_command(&mut self, pos_command: &[&str]) {
        match pos_command {
            ["position", "startpos", ..] => self.board = Board::starting_position(),
            _ => {},
        }
    }
}

impl Default for Bot {
    fn default() -> Self {
        Bot::new()
    }
}
