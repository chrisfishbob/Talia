#![allow(unused)]
use core::fmt;

use crate::board::Board;
use crate::piece::{Piece, PieceKind};
use crate::square::Square;

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
            // TODO: There is probably something more idiomatic here
            if piece.is_none() || piece.unwrap().color != self.board.to_move {
                continue;
            }

            let piece = piece.expect("Piece should not be None");
            if piece.is_sliding_piece() {
                self.generate_sliding_moves(square, piece);
            }
        }

        moves
    }

    fn generate_sliding_moves(&mut self, start_square: usize, piece: Piece) {
        let start_direction_index = if piece.piece_kind == PieceKind::Bishop {
            4
        } else {
            0
        };
        let end_direction_index = if piece.piece_kind == PieceKind::Rook {
            4
        } else {
            8
        };

        for direction_index in start_direction_index..end_direction_index {
            for n in 0..self.num_squares_to_edge[start_square][direction_index] {
                let target_square = start_square as isize
                    + self.direction_offsets[direction_index] * (n as isize + 1);
                let target_square = target_square as usize;
                let piece_on_target_square = self.board.squares[target_square];

                match piece_on_target_square {
                    Some(Piece { piece_kind, color }) => {
                        // If the piece is the opponent's color, then the capture is legal
                        // but stop here.
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
    use crate::move_generation::MoveGenerator;
    use crate::square::Square;

    #[test]
    fn test_num_squares_to_edge() {
        let board = Board::default();
        let move_generator: MoveGenerator = MoveGenerator::new(board);
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
}
