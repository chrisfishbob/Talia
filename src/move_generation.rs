#![allow(unused)]
use core::fmt;
use std::collections::HashSet;

use crate::board::Board;
use crate::piece::Piece;
use crate::square::Square;

#[derive(Eq, PartialEq, Hash)]
pub struct Move {
    pub starting_square: usize,
    pub target_square: usize,
}

impl Move {
    pub fn new(starting_square: usize, target_square: usize) -> Self {
        Self {
            starting_square,
            target_square,
        }
    }

    pub fn from_square(starting_square: Square, target_square: Square) -> Self {
        Self {
            starting_square: starting_square as usize,
            target_square: target_square as usize,
        }
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "starting_square: {:?}, target_square: {:?}",
            Square::from_index(self.starting_square),
            Square::from_index(self.target_square)
        )
    }
}

pub struct MoveGenerator {
    num_squares_to_edge: [[usize; 8]; 64],
    direction_offsets: [isize; 8],
    pub moves: Vec<Move>,
    board: Board,
}

impl Default for MoveGenerator {
    fn default() -> Self {
        Self::new(Board::starting_position())
    }
}

impl MoveGenerator {
    pub fn new(board: Board) -> Self {
        Self {
            direction_offsets: [8, -8, -1, 1, 7, -7, 9, -9],
            num_squares_to_edge: Self::precompute_move_data(),
            moves: Vec::new(),
            board,
        }
    }

    pub fn generate_moves(&mut self) -> Vec<Move> {
        let moves: Vec<Move> = Vec::new();

        for square in 0..64 {
            let piece = self.board.squares[square];
            let color = self.board.colors[square];
            match color {
                None => continue,
                Some(color) if color != self.board.to_move => continue,
                _ => (),
            }

            let piece = piece.expect("Piece should not be None if color exists");
            if piece.is_sliding_piece() {
                self.generate_sliding_moves(square);
            }
        }

        moves
    }

    fn generate_sliding_moves(&mut self, start_square: usize) {
        let piece = self.board.squares[start_square]
            .expect("should not be generating sliding moves from an empty square");
        let color = self.board.colors[start_square]
            .expect("should not be generating sliding moves from an empty square");

        let start_direction_index = if piece == Piece::Bishop { 4 } else { 0 };
        let end_direction_index = if piece == Piece::Rook { 4 } else { 8 };

        for direction_index in start_direction_index..end_direction_index {
            for n in 0..self.num_squares_to_edge[start_square][direction_index] {
                let target_square = start_square as isize
                    + self.direction_offsets[direction_index] * (n as isize + 1);
                let target_square = target_square as usize;
                let piece_on_target_square = self.board.squares[target_square];
                let color_on_target_square = self.board.colors[target_square];

                match color_on_target_square {
                    Some(color) => {
                        if color != self.board.to_move {
                            self.moves.push(Move::new(start_square, target_square));
                        }
                        // Blocked by friendly piece, cannot go on further.
                        break;
                    }
                    None => {
                        // No piece on the current square, keep generating moves
                        self.moves.push(Move::new(start_square, target_square));
                    }
                }
            }
        }
    }

    fn precompute_move_data() -> [[usize; 8]; 64] {
        let mut num_squares_to_edge = [[0; 8]; 64];
        for file in 0..8 {
            for rank in 0..8 {
                let num_north = 7 - rank;
                let num_south = rank;
                let num_east = 7 - file;
                let num_west = file;

                let square_index = rank * 8 + file;

                num_squares_to_edge[square_index] = [
                    num_north,
                    num_south,
                    num_west,
                    num_east,
                    std::cmp::min(num_north, num_west),
                    std::cmp::min(num_south, num_east),
                    std::cmp::min(num_north, num_east),
                    std::cmp::min(num_south, num_west),
                ];
            }
        }

        num_squares_to_edge
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::move_generation::{Move, MoveGenerator};
    use crate::piece::Color;
    use crate::square::Square;
    use std::collections::HashSet;

    #[test]
    fn test_num_squares_to_edge() {
        let move_generator = MoveGenerator::default();
        // North
        assert_eq!(move_generator.num_squares_to_edge[Square::A1 as usize][0], 7);
        assert_eq!(move_generator.num_squares_to_edge[Square::A4 as usize][0], 4);
        assert_eq!(move_generator.num_squares_to_edge[Square::A8 as usize][0], 0);
        // South
        assert_eq!(move_generator.num_squares_to_edge[Square::A1 as usize][1], 0);
        assert_eq!(move_generator.num_squares_to_edge[Square::A4 as usize][1], 3);
        assert_eq!(move_generator.num_squares_to_edge[Square::A8 as usize][1], 7);
        // West
        assert_eq!(move_generator.num_squares_to_edge[Square::A4 as usize][2], 0);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4 as usize][2], 4);
        assert_eq!(move_generator.num_squares_to_edge[Square::H4 as usize][2], 7);
        // East
        assert_eq!(move_generator.num_squares_to_edge[Square::A4 as usize][3], 7);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4 as usize][3], 3);
        assert_eq!(move_generator.num_squares_to_edge[Square::H4 as usize][3], 0);
        // North West
        assert_eq!(move_generator.num_squares_to_edge[Square::A1 as usize][4], 0);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4 as usize][4], 4);
        assert_eq!(move_generator.num_squares_to_edge[Square::H1 as usize][4], 7);
        // South East
        assert_eq!(move_generator.num_squares_to_edge[Square::A1 as usize][5], 0);
        assert_eq!(move_generator.num_squares_to_edge[Square::A8 as usize][5], 7);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4 as usize][5], 3);
        // North East
        assert_eq!(move_generator.num_squares_to_edge[Square::A1 as usize][6], 7);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4 as usize][6], 3);
        assert_eq!(move_generator.num_squares_to_edge[Square::H4 as usize][6], 0);
        // South West
        assert_eq!(move_generator.num_squares_to_edge[Square::A1 as usize][7], 0);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4 as usize][7], 3);
        assert_eq!(move_generator.num_squares_to_edge[Square::H8 as usize][7], 7);
    }

    #[test]
    fn test_generate_sliding_moves_empty_white() {
        let mut move_generator = MoveGenerator::default();
        move_generator.generate_sliding_moves(Square::A1 as usize);
        move_generator.generate_sliding_moves(Square::C1 as usize);
        move_generator.generate_sliding_moves(Square::D1 as usize);
        move_generator.generate_sliding_moves(Square::F1 as usize);
        move_generator.generate_sliding_moves(Square::H1 as usize);
        assert_eq!(move_generator.moves.len(), 0);
    }

    #[test]
    fn test_generate_sliding_moves_empty_black() {
        let mut move_generator = MoveGenerator::default();
        move_generator
            .board
            .move_piece(Move::from_square(Square::E2, Square::E4));
        // TODO: Remove this when move_piece handles this
        move_generator.board.to_move = Color::Black;

        move_generator.generate_sliding_moves(Square::A8 as usize);
        move_generator.generate_sliding_moves(Square::C8 as usize);
        move_generator.generate_sliding_moves(Square::D8 as usize);
        move_generator.generate_sliding_moves(Square::F8 as usize);
        move_generator.generate_sliding_moves(Square::H8 as usize);
        assert_eq!(move_generator.moves.len(), 0);
    }

    #[test]
    fn test_generate_sliding_moves_from_e4_e5() {
        let mut move_generator = MoveGenerator::default();
        move_generator
            .board
            .move_piece(Move::from_square(Square::E2, Square::E4));
        move_generator
            .board
            .move_piece(Move::from_square(Square::E7, Square::E5));

        move_generator.generate_sliding_moves(Square::A1 as usize);
        move_generator.generate_sliding_moves(Square::C1 as usize);
        move_generator.generate_sliding_moves(Square::D1 as usize);
        move_generator.generate_sliding_moves(Square::F1 as usize);
        move_generator.generate_sliding_moves(Square::H1 as usize);
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::D1, Square::E2)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::D1, Square::F3)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::D1, Square::G4)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::D1, Square::H5)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::F1, Square::E2)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::F1, Square::D3)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::F1, Square::C4)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::F1, Square::B5)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::F1, Square::A6)));
        assert_eq!(move_generator.moves.len(), 9);
    }

    #[test]
    fn test_generate_sliding_moves_from_e4_e5_nf3() {
        let mut move_generator = MoveGenerator::default();
        move_generator
            .board
            .move_piece(Move::from_square(Square::E2, Square::E4));
        move_generator
            .board
            .move_piece(Move::from_square(Square::E7, Square::E5));
        move_generator
            .board
            .move_piece(Move::from_square(Square::G1, Square::F3));
        // TODO: Remove this when move_piece handles this
        move_generator.board.to_move = Color::Black;

        move_generator.generate_sliding_moves(Square::A8 as usize);
        move_generator.generate_sliding_moves(Square::C8 as usize);
        move_generator.generate_sliding_moves(Square::D8 as usize);
        move_generator.generate_sliding_moves(Square::F8 as usize);
        move_generator.generate_sliding_moves(Square::H8 as usize);
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::D8, Square::E7)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::D8, Square::F6)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::D8, Square::G5)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::D8, Square::H4)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::F8, Square::E7)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::F8, Square::D6)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::F8, Square::C5)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::F8, Square::B4)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::F8, Square::A3)));
        assert_eq!(move_generator.moves.len(), 9);
    }

    #[test]
    fn test_generate_sliding_moves_from_e4_e5_nf3_nc6() {
        let mut move_generator = MoveGenerator::default();
        move_generator
            .board
            .move_piece(Move::from_square(Square::E2, Square::E4));
        move_generator
            .board
            .move_piece(Move::from_square(Square::E7, Square::E5));
        move_generator
            .board
            .move_piece(Move::from_square(Square::G1, Square::F3));
        move_generator
            .board
            .move_piece(Move::from_square(Square::B8, Square::C6));

        move_generator.generate_sliding_moves(Square::A1 as usize);
        move_generator.generate_sliding_moves(Square::C1 as usize);
        move_generator.generate_sliding_moves(Square::D1 as usize);
        move_generator.generate_sliding_moves(Square::F1 as usize);
        move_generator.generate_sliding_moves(Square::H1 as usize);

        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::D1, Square::E2)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::F1, Square::E2)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::F1, Square::D3)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::F1, Square::C4)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::F1, Square::B5)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::F1, Square::A6)));
        assert!(move_generator
            .moves
            .contains(&Move::from_square(Square::H1, Square::G1)));
        assert_eq!(move_generator.moves.len(), 7);
    }
}
