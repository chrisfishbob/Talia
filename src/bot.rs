use std::io::prelude::*;
use std::{fs::OpenOptions};

use crate::{
    board::Board,
    board_builder::BoardBuilder,
    move_generation::{Move, MoveGenerator},
    piece::Color,
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
            ["uci"] => respond("uciok"),
            ["isready"] => respond("readyok"),
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
            ["position", "startpos", "moves", moves @ ..] => {
                self.board = Board::starting_position();
                self.play_moves_on_board(moves);

                Ok(())
            }
            ["position", "startpos"] => {
                self.board = Board::starting_position();
                Ok(())
            }
            ["position", "fen", fen_0, fen_1, fen_2, fen_3, fen_4, fen_5, "moves", moves @ ..] => {
                let full_fen_string =
                    format!("{} {} {} {} {} {}", fen_0, fen_1, fen_2, fen_3, fen_4, fen_5);

                self.board = BoardBuilder::try_from_fen(&full_fen_string)?;
                self.play_moves_on_board(moves);

                Ok(())
            }
            ["position", "fen", fen_0, fen_1, fen_2, fen_3, fen_4, fen_5] => {
                let full_fen_string =
                    format!("{} {} {} {} {} {}", fen_0, fen_1, fen_2, fen_3, fen_4, fen_5);

                self.board = BoardBuilder::try_from_fen(&full_fen_string)?;
                Ok(())
            }
            _ => bail!("position command is in an unknown format"),
        }
    }

    fn choose_search_time_ms(
        &self,
        move_time: Option<u128>,
        time_left_on_clock: Option<u128>,
    ) -> u128 {
        match (move_time, time_left_on_clock) {
            (None, None) => 3000,
            (Some(move_time), None) => move_time,
            (None, Some(time_left_on_clock)) => self.decide_move_time(time_left_on_clock),
            (Some(_), Some(_)) => panic!("encountered invalid search time options past validation"),
        }
    }

    fn decide_move_time(&self, time_left_on_clock: u128) -> u128 {
        let is_opening_phase = self.board.full_move_number < 10;
        match is_opening_phase {
            true => time_left_on_clock / 60,
            false => time_left_on_clock / 30,
        }
    }

    fn handle_go_command(&mut self, _go_command: &[&str]) -> Result<()> {
        let mut move_generator = MoveGenerator::new(self.board.clone());
        let mut moves = move_generator.generate_moves();
        let engine_time_id = if self.board.to_move == Color::White {
            "wtime"
        } else {
            "btime"
        };
        let move_time: Option<u128> = match _go_command
            .iter()
            .position(|command| *command == "movetime")
        {
            None => None,
            Some(index) => Some(_go_command[index + 1].parse().unwrap()),
        };
        let time_left_on_clock: Option<u128> = match _go_command
            .iter()
            .position(|command| *command == engine_time_id)
        {
            None => None,
            Some(index) => Some(_go_command[index + 1].parse().unwrap()),
        };

        let (best_move, _) = find_best_move(
            &mut moves,
            &mut move_generator,
            self.choose_search_time_ms(move_time, time_left_on_clock),
        );
        self.board.move_piece(&best_move);

        respond(&format!("bestmove {best_move}"));

        Ok(())
    }

    fn play_moves_on_board(&mut self, moves: &[&str]) {
        for mv in moves {
            // Need a move generator to check if the move is legal
            let mut move_generator = MoveGenerator::new(self.board.clone());
            let mv = Move::try_from_uci(mv, &mut move_generator).unwrap();
            self.board.move_piece(&mv);
        }
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

pub fn respond(data: &str) {
    println!("{data}");
    log(data);
}

pub fn log(data: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/talia.log")
        .expect("Unable to open file");

    writeln!(file, "{data}").expect("Unable to write to log file");
}

impl Default for Bot {
    fn default() -> Self {
        Bot::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::Board,
        board_builder::BoardBuilder,
        bot::Bot,
        move_generation::{Flag, Move},
        square::Square,
    };

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

        let expected_board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::E2, Square::E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::E7, Square::E5, Flag::PawnDoublePush))
            .try_into()
            .unwrap();

        assert!(bot.board == expected_board)
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

        let expected_board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::E2, Square::E4, Flag::PawnDoublePush))
            .try_into()
            .unwrap();

        assert!(bot.board == expected_board);
    }
}
