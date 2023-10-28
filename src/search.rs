use crate::{
    evaluate::evaluate,
    move_generation::{Move, MoveGenerator},
};

// TODO: Use some infinity value instead
const CHECK_MATE_EVAL: i32 = 9999;

pub fn search(move_generator: &mut MoveGenerator, depth: u32) -> i32 {
    if depth == 0 {
        return evaluate(move_generator);
    }

    let moves = move_generator.generate_moves();
    if moves.is_empty() {
        if move_generator.is_in_check(move_generator.board.to_move) {
            return -CHECK_MATE_EVAL;
        } else {
            return 0;
        }
    }

    let mut best_eval = -CHECK_MATE_EVAL;

    for mv in moves.iter() {
        move_generator.board.move_piece(mv);
        let eval = -search(move_generator, depth - 1);
        best_eval = std::cmp::max(eval, best_eval);
        move_generator.board.unmake_move(mv).unwrap();
    }

    best_eval
}

pub fn find_best_move(move_generator: &mut MoveGenerator, depth: u32) -> Option<Move> {
    let moves = move_generator.generate_moves();
    let mut best_move = None;
    let mut best_eval = -CHECK_MATE_EVAL;

    for mv in moves {
        move_generator.board.move_piece(&mv);
        let eval = -search(move_generator, depth);
        if eval >= best_eval {
            best_eval = eval;
            best_move = Some(mv.clone());
        }
        move_generator.board.unmake_move(&mv).unwrap();
    }

    best_move
}

#[cfg(test)]
mod tests {
    use crate::{
        board::Board,
        board_builder::BoardBuilder,
        errors::BoardError,
        move_generation::{Flag, Move, MoveGenerator},
        piece::{Color, Piece},
        square::Square,
    };

    use super::find_best_move;

    #[test]
    fn test_find_best_move_mate_in_one() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H3, Piece::King, Color::Black)
            .piece(Square::A8, Piece::Rook, Color::Black)
            .to_move(Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let best_move = find_best_move(&mut move_generator, 1).unwrap();
        let mating_move = Move::from_square(Square::A8, Square::A1, Flag::None);

        assert!(best_move == mating_move);

        Ok(())
    }

    #[test]
    fn test_captures_handing_queen() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::A8, Piece::King, Color::Black)
            .piece(Square::E1, Piece::Rook, Color::White)
            .piece(Square::E5, Piece::Queen, Color::Black)
            .to_move(Color::White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let best_move = find_best_move(&mut move_generator, 1).unwrap();
        let capture_move = Move::from_square(Square::E1, Square::E5, Flag::Capture(Piece::Queen));

        assert!(best_move == capture_move);

        Ok(())
    }

    #[test]
    fn test_pins_queen_to_king() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::new()
            .piece(Square::F1, Piece::King, Color::White)
            .piece(Square::E8, Piece::King, Color::Black)
            .piece(Square::E5, Piece::Queen, Color::Black)
            .piece(Square::A1, Piece::Rook, Color::White)
            .to_move(Color::White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let best_move = find_best_move(&mut move_generator, 2).unwrap();
        let capture_move = Move::from_square(Square::A1, Square::E1, Flag::None);

        assert!(best_move == capture_move);

        Ok(())
    }

    #[test]
    fn test_forks_king_and_queen() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::new()
            .piece(Square::F1, Piece::King, Color::White)
            .piece(Square::C4, Piece::King, Color::Black)
            .piece(Square::G4, Piece::Queen, Color::Black)
            .piece(Square::D1, Piece::Knight, Color::White)
            .to_move(Color::White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let best_move = find_best_move(&mut move_generator, 2).unwrap();
        let forking_move = Move::from_square(Square::D1, Square::E3, Flag::None);

        assert!(best_move == forking_move);

        Ok(())
    }
}
