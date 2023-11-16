use std::sync::atomic::Ordering;

use crate::{
    board::Board,
    board_builder::BoardBuilder,
    errors::BoardError,
    move_generation::{Move, MoveGenerator},
    piece::Color,
    search::{find_best_move, COUNTER},
};

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

            let mut moves = move_generator.generate_moves();
            if moves.is_empty() {
                if move_generator.is_in_check(board.to_move) {
                    println!("CHECKMATE!!!!!");
                } else {
                    println!("Stalemate");
                }
                return Ok(());
            }

            println!("Talia is thinking ...");
            let start_time = std::time::Instant::now();
            let (best_move, best_eval) = find_best_move(&mut moves, &mut move_generator, engine_search_depth);
            let end_time = std::time::Instant::now();
            let elapsed_time = end_time.duration_since(start_time).as_millis();
            println!(
                "Talia thought for {} milliseconds and evaluted {} positions at depth {}",
                elapsed_time,
                COUNTER.load(Ordering::Relaxed),
                engine_search_depth
            );

            println!("Best move: {:?}", best_move);
            board.move_piece(&best_move);
            println!("Eval: {best_eval}")
        }
    }
}

fn get_uci_move_input() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("failed to read line");

    input.trim().to_owned()
}
