use std::sync::atomic::Ordering;

use crate::{
    board::Board,
    board_builder::BoardBuilder,
    errors::BoardError,
    move_generation::{Move, MoveGenerator},
    piece::Color,
    search::{find_best_move, COUNTER},
};

enum GameState {
    Active,
    Checkmate,
    Stalemate,
}

pub fn start_new_game(
    fen: Option<&str>,
    player_color: Option<Color>,
    engine_search_depth: u32,
) -> Result<(), BoardError> {
    let mut board: Board = match fen {
        None => BoardBuilder::from_starting_position().try_into()?,
        Some(fen) => BoardBuilder::try_from_fen(fen)?,
    };

    loop {
        let mut move_generator = MoveGenerator::new(board.clone());
        match check_game_state(&mut move_generator) {
            GameState::Active => {}
            GameState::Checkmate => {
                println!("Checkmate!");
                return Ok(());
            }
            GameState::Stalemate => {
                println!("Stalemate!");
                return Ok(());
            }
        }

        if player_color.is_some_and(|color| color == board.to_move) {
            println!("{}", board);
            let input = get_uci_move_input();
            match Move::try_from_algebraic_notation(&input, &mut move_generator) {
                Ok(mv) => board.move_piece(&mv),
                Err(error) => println!("{}", error),
            }
        } else {
            // Talia plays
            // Only print the board while Talia is thinking if there is no human player
            if player_color.is_none() {
                println!("{}", board);
            }

            match check_game_state(&mut move_generator) {
                GameState::Active => {}
                GameState::Checkmate => {
                    println!("Checkmate!");
                    return Ok(());
                }
                GameState::Stalemate => {
                    println!("Stalemate!");
                    return Ok(());
                }
            }

            println!("Talia is thinking ...");
            let start_time = std::time::Instant::now();
            let (best_move, mut best_eval) = find_best_move(
                &mut move_generator.generate_moves(),
                &mut move_generator,
                engine_search_depth,
            );
            let end_time = std::time::Instant::now();
            let elapsed_time = end_time.duration_since(start_time).as_millis();
            println!(
                "Talia thought for {} milliseconds and evaluted {} positions at depth {}",
                elapsed_time,
                COUNTER.load(Ordering::Relaxed),
                engine_search_depth
            );

            println!("Best move: {:?}", best_move);

            // Display the eval without perspective.
            // Positive eval: white has advantage, negative eval: black has advantage
            if move_generator.board.to_move == Color::Black {
                best_eval *= -1
            }
            board.move_piece(&best_move);
            println!("Eval: {best_eval}")
        }
    }
}

fn check_game_state(move_generator: &mut MoveGenerator) -> GameState {
    let moves = move_generator.generate_moves();
    match moves.is_empty() {
        true => {
            if move_generator.is_in_check(move_generator.board.to_move) {
                GameState::Checkmate
            } else {
                GameState::Stalemate
            }
        }
        false => GameState::Active,
    }
}

fn get_uci_move_input() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("failed to read line");

    input.trim().to_owned()
}
