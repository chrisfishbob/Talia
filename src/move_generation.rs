#![allow(unused)]
use crate::board::Board;
use crate::square::Square;

pub struct Move {
    pub starting_square: Square,
    pub target_square: Square,
}

impl Move {
    pub fn new(starting_square: Square, target_square: Square) -> Self {
        Self {
            starting_square,
            target_square,
        }
    }
}

struct MoveGenerator {}

impl MoveGenerator {
    fn generate_moves(board: Board) -> Vec<Move> {
        let moves: Vec<Move> = Vec::new();
        moves
    }
}
