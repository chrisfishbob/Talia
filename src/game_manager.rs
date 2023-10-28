use crate::{
    board::Board,
    board_builder::BoardBuilder,
    errors::BoardError,
    move_generation::{Move, MoveGenerator},
    piece::Color,
    search::find_best_move,
};

pub fn start_new_game(fen: Option<&str>, player_color: Color, engine_search_depth: u32) -> Result<(), BoardError> {
    let mut board: Board = match fen {
        None => BoardBuilder::from_starting_position().try_into()?,
        Some(fen) => BoardBuilder::try_from_fen(fen)?,
    };

    loop {
        let mut move_generator = MoveGenerator::new(board.clone());
        if player_color == board.to_move {
            println!("{}", board);
            let input = get_uci_move_input();
            match Move::try_from_algebraic_notation(&input, &mut move_generator) {
                Ok(mv) => board.move_piece(&mv),
                Err(error) => println!("{}", error),
            }
        } else {
            // Talia plays
            println!("Talia is thingking ...");
            let start_time = std::time::Instant::now();
            let best_move = find_best_move(&mut move_generator, engine_search_depth);
            let end_time = std::time::Instant::now();
            let elapsed_time = end_time.duration_since(start_time).as_millis();
            println!("Talia thought for {} milliseconds at depth {}", elapsed_time, engine_search_depth);

            match best_move {
                None => {
                    if move_generator.is_in_check(board.to_move) {
                        println!("CHECKMATE!!!!!");
                    } else {
                        println!("Stalemate");
                    }
                    return Ok(());
                }
                Some(mv) => board.move_piece(&mv),
            }
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
