use crate::{move_generation::MoveGenerator, piece::Color};

pub fn evaluate(move_generator: &MoveGenerator) -> i32 {
    let mut eval = 0;
    let board = &move_generator.board;

    for square in 0..64 {
        if let Some(piece) = board.squares[square] {
            if board.colors[square].unwrap() == Color::White {
                eval += piece.piece_value() + piece.position_value(square, Color::White)
            } else {
                eval -= piece.piece_value() + piece.position_value(square, Color::Black)
            }
        }
    }

    if move_generator.board.to_move == Color::White {
        eval
    } else {
        -eval
    }
}

#[cfg(test)]
mod tests {
    use crate::{board::Board, move_generation::MoveGenerator};

    use super::evaluate;

    #[test]
    fn test_starting_position_eval() {
        let board = Board::starting_position();
        let move_generator = MoveGenerator::new(board);

        let eval = evaluate(&move_generator);
        assert!(eval == 0);
    }
}
