use std::fs::OpenOptions;
use std::io::prelude::*;

use crate::{
    board::Board, board_builder::BoardBuilder, move_generation::MoveGenerator,
    search::find_best_move,
};
use anyhow::{bail, Result};

pub struct Bot {
    board: Board,
}

impl Bot {
    pub fn new() -> Self {
        Self {
            board: Board::starting_position(),
        }
    }

    pub fn start_uci(&mut self) -> Result<()> {
        loop {
            let input = self.get_uci_move_input();
            let split_input: Vec<&str> = input.split_whitespace().collect();
            let commands = split_input.as_slice();
            self.log(&input);
            if let Err(e) = self.process_commands(commands) {
                self.log("Talia encountered a critical error");
                self.log(&e.to_string());
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

    fn process_commands(&mut self, commands: &[&str]) -> Result<()> {
        match commands {
            ["uci"] => self.respond("uciok"),
            ["isready"] => self.respond("readyok"),
            ["position", ..] => self.handle_position_command(commands)?,
            ["go", ..] => self.handle_go_command(commands)?,
            // TODO: Handle stop once clock is implemented in searcher
            ["ucinewgame"] | ["stop"] => {}
            ["quit"] => std::process::exit(0),
            _ => bail!("unrecognized UCI command"),
        }
        Ok(())
    }

    fn handle_position_command(&mut self, pos_command: &[&str]) -> Result<()> {
        // Format: 'position startpos moves e2e4 e7e5'
        // Or: 'position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 moves e2e4 e7e5'
        // Note: 'moves' section is optional
        // (Thanks Sebastian for figuring this out, so I don't have to read the specs <3)
        match pos_command {
            ["position", "startpos", ..] => {
                self.board = Board::starting_position();
                Ok(())
            }
            ["position", "fen", fen_0, fen_1, fen_2, fen_3, fen_4, fen_5, ..] => {
                let full_fen_string =
                    format!("{} {} {} {} {} {}", fen_0, fen_1, fen_2, fen_3, fen_4, fen_5);
                self.board = BoardBuilder::try_from_fen(&full_fen_string)?;
                Ok(())
            }
            _ => bail!("position command is in an unknown format"),
        }
    }

    fn handle_go_command(&mut self, _go_command: &[&str]) -> Result<()> {
        // TODO: Handle time and increments
        let depth = 6;
        let mut move_generator = MoveGenerator::new(self.board.clone());
        let mut moves = move_generator.generate_moves();
        let (best_move, _) = find_best_move(&mut moves, &mut move_generator, depth);
        self.board.move_piece(&best_move);

        self.respond(&format!("bestmove {best_move}"));

        Ok(())
    }

    fn respond(&self, data: &str) {
        println!("{data}");
        self.log(data);
    }

    fn log(&self, data: &str) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/talia.log")
            .expect("Unable to open file");

        writeln!(file, "{data}").expect("Unable to write to log file");
    }
}

impl Default for Bot {
    fn default() -> Self {
        Bot::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{board::Board, bot::Bot};

    #[test]
    fn test_uci_command_position() {
        let mut bot = Bot::new();
        let command = ["position", "startpos"];
        bot.process_commands(&command).unwrap();

        assert!(bot.board == Board::starting_position())
    }

    #[test]
    fn test_uci_command_position_with_optional_moves() {
        let mut bot = Bot::new();
        let command = ["position", "startpos", "moves", "e2e4", "e7e5"];
        bot.process_commands(&command).unwrap();

        assert!(bot.board == Board::starting_position())
    }

    #[test]
    fn test_uci_command_position_with_fen() {
        let mut bot = Bot::new();
        let command = [
            "position",
            "fen",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
            "w",
            "KQkq",
            "-",
            "0",
            "1",
        ];
        bot.process_commands(&command).unwrap();

        assert!(bot.board.to_fen() == "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    #[test]
    fn test_uci_command_position_with_fen_and_moves() {
        let mut bot = Bot::new();
        let command = [
            "position",
            "fen",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
            "w",
            "KQkq",
            "-",
            "0",
            "1",
            "moves",
            "e2e4",
        ];
        bot.process_commands(&command).unwrap();

        assert!(bot.board.to_fen() == "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }
}
