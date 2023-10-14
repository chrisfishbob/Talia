use core::fmt;

use crate::board::Board;
use crate::piece::{Color, Piece};
use crate::square::Square;

#[derive(Eq, PartialEq)]
pub struct Move {
    pub starting_square: usize,
    pub target_square: usize,
    pub flag: Flag,
}

impl Move {
    pub fn new(start: usize, target: usize, flag: Flag) -> Self {
        Self {
            starting_square: start,
            target_square: target,
            flag,
        }
    }

    pub fn from_square(start: Square, target: Square, flag: Flag) -> Self {
        Self {
            starting_square: start as usize,
            target_square: target as usize,
            flag,
        }
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "starting_square: {:?}, target_square: {:?}",
            Square::from_index(self.starting_square),
            Square::from_index(self.target_square),
        )?;
        if let Flag::PromoteTo(piece) = self.flag {
            write!(f, ", promotion_piece: {:?}", piece)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Flag {
    None,
    Castle,
    PawnDoublePush,
    EnPassantCapture,
    PromoteTo(Piece),
}

pub struct MoveGenerator {
    pub moves: Vec<Move>,
    num_squares_to_edge: [[usize; 8]; 64],
    direction_offsets: [isize; 8],
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
            match piece {
                Piece::Queen | Piece::Rook | Piece::Bishop => self.generate_sliding_moves(square),
                Piece::Knight => self.generate_knight_moves(square),
                Piece::Pawn => self.generate_pawn_moves(square),
                Piece::King => self.generate_king_moves(square),
            }
        }

        moves
    }

    fn generate_sliding_moves(&mut self, start_square: usize) {
        let piece = self.board.squares[start_square]
            .expect("should not be generating sliding moves from an empty square");

        let start_direction_index = if piece == Piece::Bishop { 4 } else { 0 };
        let end_direction_index = if piece == Piece::Rook { 4 } else { 8 };

        for direction_index in start_direction_index..end_direction_index {
            for n in 0..self.num_squares_to_edge[start_square][direction_index] {
                let target_square = start_square as isize
                    + self.direction_offsets[direction_index] * (n as isize + 1);
                let target_square = target_square as usize;
                let color_on_target_square = self.board.colors[target_square];

                match color_on_target_square {
                    Some(color) => {
                        if color != self.board.to_move {
                            self.moves
                                .push(Move::new(start_square, target_square, Flag::None));
                        }
                        // Blocked by friendly piece, cannot go on further.
                        break;
                    }
                    None => {
                        // No piece on the current square, keep generating moves
                        self.moves
                            .push(Move::new(start_square, target_square, Flag::None));
                    }
                }
            }
        }
    }

    fn generate_knight_moves(&mut self, start_square: usize) {
        let knight_move_offsets = [-17, -15, -10, -6, 6, 10, 15, 17];

        for offset in knight_move_offsets {
            let target_square = {
                let tmp = start_square as isize + offset;
                if !(0..64).contains(&tmp) {
                    continue;
                }
                tmp as usize
            };

            if Self::is_pacman_move(start_square, target_square) {
                continue;
            }

            match self.board.colors[target_square] {
                None => self
                    .moves
                    .push(Move::new(start_square, target_square, Flag::None)),
                Some(color) if color != self.board.to_move => {
                    self.moves
                        .push(Move::new(start_square, target_square, Flag::None))
                }
                _ => continue,
            }
        }
    }

    fn generate_pawn_moves(&mut self, start_square: usize) {
        let pawn_move_offsets = match self.board.to_move {
            Color::White => [8, 16, 7, 9],
            Color::Black => [-8, -16, -7, -9],
        };

        let target_one_up_index = start_square as isize + pawn_move_offsets[0];
        let target_one_up_rank = target_one_up_index / 8;
        let can_move_up_one_rank = self.board.squares[target_one_up_index as usize].is_none();

        if can_move_up_one_rank {
            let target_one_up_index = target_one_up_index as usize;
            let is_promotion_move = target_one_up_rank == 0 || target_one_up_rank == 7;
            if !is_promotion_move {
                self.moves
                    .push(Move::new(start_square, target_one_up_index, Flag::None));
            } else {
                self.add_promotion_moves(start_square, target_one_up_index);
            }
        }

        for capture_offset in &pawn_move_offsets[2..] {
            let target_square = {
                let tmp = start_square as isize + capture_offset;
                if !(0..64).contains(&tmp) {
                    continue;
                }
                tmp as usize
            };

            if Self::is_pacman_move(start_square, target_square) {
                continue;
            }

            let is_occupied_by_opponent_piece =
                self.board.colors[target_square].is_some_and(|color| color != self.board.to_move);
            let can_capture_en_passant = self
                .board
                .en_passant_square
                .is_some_and(|index| index == target_square);

            if is_occupied_by_opponent_piece || can_capture_en_passant {
                let target_rank = target_square / 8;
                let is_promotion_move = target_rank == 0 || target_rank == 7;

                if is_promotion_move {
                    self.add_promotion_moves(start_square, target_square);
                } else if can_capture_en_passant {
                    self.moves
                        .push(Move::new(start_square, target_square, Flag::EnPassantCapture));
                } else {
                    self.moves
                        .push(Move::new(start_square, target_square, Flag::None));
                }
            }
        }

        // If a pawn cannot move one square up, it definitely cannot move up by two
        if !can_move_up_one_rank {
            return;
        }

        // If pawn already moved, it cannot move up by two
        let starting_rank = start_square / 8;
        let has_moved = (starting_rank != 1 && self.board.to_move == Color::White)
            || (starting_rank != 6 && self.board.to_move == Color::Black);
        if has_moved {
            return;
        }

        let target_two_up_index = start_square as isize + pawn_move_offsets[1];
        if self.board.squares[target_two_up_index as usize].is_none() {
            self.moves.push(Move::new(
                start_square,
                target_two_up_index as usize,
                Flag::PawnDoublePush,
            ));
        }
    }

    fn generate_king_moves(&mut self, start_square: usize) {
        for offset in self.direction_offsets {
            let target_square = {
                let tmp = start_square as isize + offset;
                if !(0..64).contains(&tmp) {
                    continue;
                }
                tmp as usize
            };

            if Self::is_pacman_move(start_square, target_square) {
                continue;
            }

            if self.board.colors[target_square].is_none()
                || self.board.colors[target_square]
                    .is_some_and(|color| color != self.board.colors[start_square].unwrap())
            {
                self.moves
                    .push(Move::new(start_square, target_square, Flag::None));
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

    fn add_promotion_moves(&mut self, start: usize, target: usize) {
        self.moves
            .push(Move::new(start, target, Flag::PromoteTo(Piece::Queen)));
        self.moves
            .push(Move::new(start, target, Flag::PromoteTo(Piece::Rook)));
        self.moves
            .push(Move::new(start, target, Flag::PromoteTo(Piece::Bishop)));
        self.moves
            .push(Move::new(start, target, Flag::PromoteTo(Piece::Knight)));
    }

    fn is_pacman_move(start: usize, target: usize) -> bool {
        let starting_rank = start as isize / 8;
        let starting_file = start as isize % 8;
        let target_rank = target as isize / 8;
        let target_file = target as isize % 8;

        // Prevents pieces from teleporting from one side to another Pacman-style
        // Two ranks or columns is the most a non-sliding piece can legally move
        (target_rank - starting_rank).abs() > 2 || (target_file - starting_file).abs() > 2
    }

    #[cfg(test)]
    fn generated_move(&self, start: Square, target: Square, flag: Flag) -> bool {
        self.moves.contains(&Move::from_square(start, target, flag))
    }
}

#[cfg(test)]
mod tests {
    use crate::board_builder::BoardBuilder;
    use crate::errors::BoardError;
    use crate::move_generation::{Flag, Move, MoveGenerator};
    use crate::piece::{Color, Piece};
    use crate::square::Square;

    #[test]
    fn test_num_squares_to_edge() {
        let move_generator = MoveGenerator::default();
        // North
        assert_eq!(move_generator.num_squares_to_edge[Square::A1.as_index()][0], 7);
        assert_eq!(move_generator.num_squares_to_edge[Square::A4.as_index()][0], 4);
        assert_eq!(move_generator.num_squares_to_edge[Square::A8.as_index()][0], 0);
        // South
        assert_eq!(move_generator.num_squares_to_edge[Square::A1.as_index()][1], 0);
        assert_eq!(move_generator.num_squares_to_edge[Square::A4.as_index()][1], 3);
        assert_eq!(move_generator.num_squares_to_edge[Square::A8.as_index()][1], 7);
        // West
        assert_eq!(move_generator.num_squares_to_edge[Square::A4.as_index()][2], 0);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4.as_index()][2], 4);
        assert_eq!(move_generator.num_squares_to_edge[Square::H4.as_index()][2], 7);
        // East
        assert_eq!(move_generator.num_squares_to_edge[Square::A4.as_index()][3], 7);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4.as_index()][3], 3);
        assert_eq!(move_generator.num_squares_to_edge[Square::H4.as_index()][3], 0);
        // North West
        assert_eq!(move_generator.num_squares_to_edge[Square::A1.as_index()][4], 0);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4.as_index()][4], 4);
        assert_eq!(move_generator.num_squares_to_edge[Square::H1.as_index()][4], 7);
        // South East
        assert_eq!(move_generator.num_squares_to_edge[Square::A1.as_index()][5], 0);
        assert_eq!(move_generator.num_squares_to_edge[Square::A8.as_index()][5], 7);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4.as_index()][5], 3);
        // North East
        assert_eq!(move_generator.num_squares_to_edge[Square::A1.as_index()][6], 7);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4.as_index()][6], 3);
        assert_eq!(move_generator.num_squares_to_edge[Square::H4.as_index()][6], 0);
        // South West
        assert_eq!(move_generator.num_squares_to_edge[Square::A1.as_index()][7], 0);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4.as_index()][7], 3);
        assert_eq!(move_generator.num_squares_to_edge[Square::H8.as_index()][7], 7);
    }

    #[test]
    fn test_generate_sliding_moves_empty_white() {
        let mut move_generator = MoveGenerator::default();
        move_generator.generate_sliding_moves(Square::A1.as_index());
        move_generator.generate_sliding_moves(Square::C1.as_index());
        move_generator.generate_sliding_moves(Square::D1.as_index());
        move_generator.generate_sliding_moves(Square::F1.as_index());
        move_generator.generate_sliding_moves(Square::H1.as_index());
        assert_eq!(move_generator.moves.len(), 0);
    }

    #[test]
    fn test_generate_sliding_moves_empty_black() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::E2, Square::E4, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);

        move_generator.generate_sliding_moves(Square::A8.as_index());
        move_generator.generate_sliding_moves(Square::C8.as_index());
        move_generator.generate_sliding_moves(Square::D8.as_index());
        move_generator.generate_sliding_moves(Square::F8.as_index());
        move_generator.generate_sliding_moves(Square::H8.as_index());
        assert_eq!(move_generator.moves.len(), 0);
        Ok(())
    }

    #[test]
    fn test_generate_sliding_moves_from_e4_e5() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::E2, Square::E4, Flag::None))
            .make_move(Move::from_square(Square::E7, Square::E5, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);

        move_generator.generate_sliding_moves(Square::A1.as_index());
        move_generator.generate_sliding_moves(Square::C1.as_index());
        move_generator.generate_sliding_moves(Square::D1.as_index());
        move_generator.generate_sliding_moves(Square::F1.as_index());
        move_generator.generate_sliding_moves(Square::H1.as_index());

        assert!(move_generator.generated_move(Square::D1, Square::E2, Flag::None));
        assert!(move_generator.generated_move(Square::D1, Square::F3, Flag::None));
        assert!(move_generator.generated_move(Square::D1, Square::G4, Flag::None));
        assert!(move_generator.generated_move(Square::D1, Square::H5, Flag::None));
        assert!(move_generator.generated_move(Square::F1, Square::E2, Flag::None));
        assert!(move_generator.generated_move(Square::F1, Square::D3, Flag::None));
        assert!(move_generator.generated_move(Square::F1, Square::C4, Flag::None));
        assert!(move_generator.generated_move(Square::F1, Square::B5, Flag::None));
        assert!(move_generator.generated_move(Square::F1, Square::A6, Flag::None));
        assert_eq!(move_generator.moves.len(), 9);
        Ok(())
    }

    #[test]
    fn test_generate_sliding_moves_from_e4_e5_nf3() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::E2, Square::E4, Flag::None))
            .make_move(Move::from_square(Square::E7, Square::E5, Flag::None))
            .make_move(Move::from_square(Square::G1, Square::F3, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);

        move_generator.generate_sliding_moves(Square::A8.as_index());
        move_generator.generate_sliding_moves(Square::C8.as_index());
        move_generator.generate_sliding_moves(Square::D8.as_index());
        move_generator.generate_sliding_moves(Square::F8.as_index());
        move_generator.generate_sliding_moves(Square::H8.as_index());

        assert!(move_generator.generated_move(Square::D8, Square::E7, Flag::None));
        assert!(move_generator.generated_move(Square::D8, Square::F6, Flag::None));
        assert!(move_generator.generated_move(Square::D8, Square::G5, Flag::None));
        assert!(move_generator.generated_move(Square::D8, Square::H4, Flag::None));
        assert!(move_generator.generated_move(Square::F8, Square::E7, Flag::None));
        assert!(move_generator.generated_move(Square::F8, Square::D6, Flag::None));
        assert!(move_generator.generated_move(Square::F8, Square::C5, Flag::None));
        assert!(move_generator.generated_move(Square::F8, Square::B4, Flag::None));
        assert!(move_generator.generated_move(Square::F8, Square::A3, Flag::None));
        assert_eq!(move_generator.moves.len(), 9);

        Ok(())
    }

    #[test]
    fn test_generate_sliding_moves_from_e4_e5_nf3_nc6() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::E2, Square::E4, Flag::None))
            .make_move(Move::from_square(Square::E7, Square::E5, Flag::None))
            .make_move(Move::from_square(Square::G1, Square::F3, Flag::None))
            .make_move(Move::from_square(Square::B8, Square::C6, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);

        move_generator.generate_sliding_moves(Square::A1.as_index());
        move_generator.generate_sliding_moves(Square::C1.as_index());
        move_generator.generate_sliding_moves(Square::D1.as_index());
        move_generator.generate_sliding_moves(Square::F1.as_index());
        move_generator.generate_sliding_moves(Square::H1.as_index());

        assert!(move_generator.generated_move(Square::D1, Square::E2, Flag::None));
        assert!(move_generator.generated_move(Square::F1, Square::E2, Flag::None));
        assert!(move_generator.generated_move(Square::F1, Square::D3, Flag::None));
        assert!(move_generator.generated_move(Square::F1, Square::C4, Flag::None));
        assert!(move_generator.generated_move(Square::F1, Square::B5, Flag::None));
        assert!(move_generator.generated_move(Square::F1, Square::A6, Flag::None));
        assert!(move_generator.generated_move(Square::H1, Square::G1, Flag::None));
        assert_eq!(move_generator.moves.len(), 7);

        Ok(())
    }

    #[test]
    fn test_generate_sliding_moves_from_corner() -> Result<(), BoardError> {
        let board = BoardBuilder::try_from_fen("Qr5k/r7/2N5/8/8/8/8/6K1 w - - 0 1")?;
        let mut move_generator = MoveGenerator::new(board);

        move_generator.generate_sliding_moves(Square::A8.as_index());

        assert_eq!(move_generator.moves.len(), 3);
        assert!(move_generator.generated_move(Square::A8, Square::A7, Flag::None));
        assert!(move_generator.generated_move(Square::A8, Square::B8, Flag::None));
        assert!(move_generator.generated_move(Square::A8, Square::B7, Flag::None));

        Ok(())
    }

    #[test]
    fn test_generate_knight_moves_starting_position() {
        let mut move_generator = MoveGenerator::default();
        move_generator.generate_knight_moves(Square::B1.as_index());
        move_generator.generate_knight_moves(Square::G1.as_index());

        assert_eq!(move_generator.moves.len(), 4);
        assert!(move_generator.generated_move(Square::B1, Square::A3, Flag::None));
        assert!(move_generator.generated_move(Square::B1, Square::A3, Flag::None));
        assert!(move_generator.generated_move(Square::B1, Square::C3, Flag::None));
        assert!(move_generator.generated_move(Square::G1, Square::F3, Flag::None));
        assert!(move_generator.generated_move(Square::G1, Square::H3, Flag::None));
    }

    #[test]
    fn test_generate_knight_moves_from_corner() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::A1, Piece::King, Color::White)
            .piece(Square::A1, Piece::King, Color::White)
            .piece(Square::B1, Piece::Rook, Color::White)
            .piece(Square::H1, Piece::Knight, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_knight_moves(Square::H1.as_index());

        assert_eq!(move_generator.moves.len(), 2);
        assert!(move_generator.generated_move(Square::H1, Square::F2, Flag::None));
        assert!(move_generator.generated_move(Square::H1, Square::G3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_generate_knight_moves_from_near_corner() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::A1, Piece::King, Color::White)
            .piece(Square::A1, Piece::King, Color::White)
            .piece(Square::B1, Piece::Rook, Color::White)
            .piece(Square::G2, Piece::Knight, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_knight_moves(Square::G2.as_index());

        assert_eq!(move_generator.moves.len(), 4);
        assert!(move_generator.generated_move(Square::G2, Square::E1, Flag::None));
        assert!(move_generator.generated_move(Square::G2, Square::E3, Flag::None));
        assert!(move_generator.generated_move(Square::G2, Square::F4, Flag::None));
        assert!(move_generator.generated_move(Square::G2, Square::H4, Flag::None));

        Ok(())
    }

    #[test]
    fn test_generate_knight_moves_with_pieces_on_target_square() -> Result<(), BoardError> {
        let board = BoardBuilder::try_from_fen("k7/3R1n2/2n3R1/4N3/2R3n1/3n1R2/8/KR6 w - - 0 1")?;
        let mut move_generator = MoveGenerator::new(board);

        move_generator.generate_knight_moves(Square::E5.as_index());
        assert_eq!(move_generator.moves.len(), 4);
        assert!(move_generator.generated_move(Square::E5, Square::C6, Flag::None));
        assert!(move_generator.generated_move(Square::E5, Square::D3, Flag::None));
        assert!(move_generator.generated_move(Square::E5, Square::G4, Flag::None));
        assert!(move_generator.generated_move(Square::E5, Square::F7, Flag::None));

        Ok(())
    }

    #[test]
    fn test_generate_pawn_moves_from_starting_position_white() {
        let mut move_generator = MoveGenerator::default();

        for square in 0..64 {
            if move_generator.board.is_piece_at_square(
                square,
                Piece::Pawn,
                move_generator.board.to_move,
            ) {
                move_generator.generate_pawn_moves(square);
            }
        }

        assert_eq!(move_generator.moves.len(), 16);
        assert!(move_generator.generated_move(Square::A2, Square::A3, Flag::None));
        assert!(move_generator.generated_move(Square::A2, Square::A4, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(Square::B2, Square::B3, Flag::None));
        assert!(move_generator.generated_move(Square::B2, Square::B4, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(Square::C2, Square::C3, Flag::None));
        assert!(move_generator.generated_move(Square::C2, Square::C4, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(Square::D2, Square::D3, Flag::None));
        assert!(move_generator.generated_move(Square::D2, Square::D4, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(Square::E2, Square::E3, Flag::None));
        assert!(move_generator.generated_move(Square::E2, Square::E4, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(Square::F2, Square::F3, Flag::None));
        assert!(move_generator.generated_move(Square::F2, Square::F4, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(Square::G2, Square::G3, Flag::None));
        assert!(move_generator.generated_move(Square::G2, Square::G4, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(Square::H2, Square::H3, Flag::None));
        assert!(move_generator.generated_move(Square::H2, Square::H4, Flag::PawnDoublePush));
    }

    #[test]
    fn test_generate_pawn_moves_from_starting_position_black() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::E2, Square::E4, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);

        for square in 0..64 {
            if move_generator.board.is_piece_at_square(
                square,
                Piece::Pawn,
                move_generator.board.to_move,
            ) {
                move_generator.generate_pawn_moves(square);
            }
        }

        assert_eq!(move_generator.moves.len(), 16);
        assert!(move_generator.generated_move(Square::A7, Square::A5, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(Square::A7, Square::A6, Flag::None));
        assert!(move_generator.generated_move(Square::B7, Square::B5, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(Square::B7, Square::B6, Flag::None));
        assert!(move_generator.generated_move(Square::C7, Square::C5, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(Square::C7, Square::C6, Flag::None));
        assert!(move_generator.generated_move(Square::D7, Square::D5, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(Square::D7, Square::D6, Flag::None));
        assert!(move_generator.generated_move(Square::E7, Square::E5, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(Square::E7, Square::E6, Flag::None));
        assert!(move_generator.generated_move(Square::F7, Square::F5, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(Square::F7, Square::F6, Flag::None));
        assert!(move_generator.generated_move(Square::G7, Square::G5, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(Square::G7, Square::G6, Flag::None));
        assert!(move_generator.generated_move(Square::H7, Square::H5, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(Square::H7, Square::H6, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_move_with_piece_blocking_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            // Tests that opposite color pieces block movement
            .piece(Square::F4, Piece::Pawn, Color::White)
            .piece(Square::F5, Piece::Knight, Color::Black)
            // Tests that same color pieces also block movement
            .piece(Square::C4, Piece::Pawn, Color::White)
            .piece(Square::C5, Piece::Knight, Color::White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::F4.as_index());
        move_generator.generate_pawn_moves(Square::C4.as_index());

        assert_eq!(move_generator.moves.len(), 0);

        Ok(())
    }

    #[test]
    fn test_pawn_move_with_piece_blocking_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            // Tests that opposite color pieces block movement
            .piece(Square::F5, Piece::Pawn, Color::Black)
            .piece(Square::F4, Piece::Knight, Color::White)
            // Tests that same color pieces also block movement
            .piece(Square::C5, Piece::Pawn, Color::Black)
            .piece(Square::C4, Piece::Knight, Color::Black)
            .to_move(Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::F5.as_index());
        move_generator.generate_pawn_moves(Square::C5.as_index());

        assert_eq!(move_generator.moves.len(), 0);

        Ok(())
    }

    #[test]
    fn test_pawn_with_second_rank_blocked_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .piece(Square::E2, Piece::Pawn, Color::White)
            .piece(Square::E4, Piece::Pawn, Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::E2.as_index());

        assert_eq!(move_generator.moves.len(), 1);
        assert!(move_generator.generated_move(Square::E2, Square::E3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_with_second_rank_blocked_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .piece(Square::E7, Piece::Pawn, Color::Black)
            .piece(Square::E5, Piece::Pawn, Color::White)
            .to_move(Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::E7.as_index());

        assert_eq!(move_generator.moves.len(), 1);
        assert!(move_generator.generated_move(Square::E7, Square::E6, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_both_captures_in_center_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .piece(Square::D5, Piece::Pawn, Color::Black)
            .piece(Square::E4, Piece::Pawn, Color::White)
            .piece(Square::E5, Piece::Pawn, Color::White)
            .piece(Square::F5, Piece::Pawn, Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::E4.as_index());

        assert_eq!(move_generator.moves.len(), 2);
        assert!(move_generator.generated_move(Square::E4, Square::D5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::F5, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_both_captures_in_center_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .piece(Square::D4, Piece::Pawn, Color::White)
            .piece(Square::E5, Piece::Pawn, Color::Black)
            .piece(Square::E4, Piece::Pawn, Color::Black)
            .piece(Square::F4, Piece::Pawn, Color::White)
            .to_move(Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::E5.as_index());

        assert_eq!(move_generator.moves.len(), 2);
        assert!(move_generator.generated_move(Square::E5, Square::F4, Flag::None));
        assert!(move_generator.generated_move(Square::E5, Square::D4, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_no_pacman_white() -> Result<(), BoardError> {
        // If pacman behavior exists, a capture offset of 9 for a pawn at the
        // 7th file will result in a square in the 0th file to become the target
        // square.
        let board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .piece(Square::H4, Piece::Pawn, Color::White)
            .piece(Square::G5, Piece::Pawn, Color::Black)
            // If the pacman behavior exists, the A6 pawn would be a target square
            .piece(Square::A6, Piece::Pawn, Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::H4.as_index());

        assert_eq!(move_generator.moves.len(), 2);
        assert!(move_generator.generated_move(Square::H4, Square::G5, Flag::None));
        assert!(move_generator.generated_move(Square::H4, Square::H5, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_no_pacman_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .piece(Square::A5, Piece::Pawn, Color::Black)
            .piece(Square::B4, Piece::Pawn, Color::White)
            // If anti-pacman behavior exists, the H3 pawn would be a target square
            .piece(Square::H3, Piece::Pawn, Color::White)
            .to_move(Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::A5.as_index());

        assert_eq!(move_generator.moves.len(), 2);
        assert!(move_generator.generated_move(Square::A5, Square::B4, Flag::None));
        assert!(move_generator.generated_move(Square::A5, Square::A4, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_no_anti_pacman_white() -> Result<(), BoardError> {
        // If anti-pacman behavior exists, a capture offset for a pawn at the 0th
        // file will result in the square on the 8th file on the same rank to become
        // the target square.
        let board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .piece(Square::A3, Piece::Pawn, Color::White)
            // If the pacman behavior exists, the H3 pawn would be a target square
            .piece(Square::H3, Piece::Pawn, Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::A3.as_index());

        assert_eq!(move_generator.moves.len(), 1);
        assert!(move_generator.generated_move(Square::A3, Square::A4, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_no_anti_pacman_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .piece(Square::H5, Piece::Pawn, Color::Black)
            .piece(Square::A5, Piece::Pawn, Color::White)
            .to_move(Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::H5.as_index());

        assert_eq!(move_generator.moves.len(), 1);
        assert!(move_generator.generated_move(Square::H5, Square::H4, Flag::None));

        Ok(())
    }

    #[test]
    fn test_already_moved_pawn_white() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::E2, Square::E4, Flag::None))
            .make_move(Move::from_square(Square::G8, Square::F6, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::E4.as_index());

        assert_eq!(move_generator.moves.len(), 1);
        assert!(move_generator.generated_move(Square::E4, Square::E5, Flag::None));

        Ok(())
    }

    #[test]
    fn test_already_moved_pawn_black() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::H2, Square::H4, Flag::None))
            .make_move(Move::from_square(Square::E7, Square::E5, Flag::None))
            .make_move(Move::from_square(Square::H4, Square::H5, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::E5.as_index());

        assert_eq!(move_generator.moves.len(), 1);
        assert!(move_generator.generated_move(Square::E5, Square::E4, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_capture_index_no_overflow() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::H7, Piece::Pawn, Color::White)
            .piece(Square::E1, Piece::King, Color::White)
            .piece(Square::E8, Piece::King, Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::H7.as_index());

        assert!(move_generator.moves.len() == 4);
        assert!(move_generator.generated_move(
            Square::H7,
            Square::H8,
            Flag::PromoteTo(Piece::Queen)
        ));
        assert!(move_generator.generated_move(
            Square::H7,
            Square::H8,
            Flag::PromoteTo(Piece::Rook)
        ));
        assert!(move_generator.generated_move(
            Square::H7,
            Square::H8,
            Flag::PromoteTo(Piece::Bishop)
        ));
        assert!(move_generator.generated_move(
            Square::H7,
            Square::H8,
            Flag::PromoteTo(Piece::Knight)
        ));

        Ok(())
    }

    #[test]
    fn test_pawn_capture_index_no_underflow() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::A2, Piece::Pawn, Color::Black)
            .piece(Square::E1, Piece::King, Color::White)
            .piece(Square::E8, Piece::King, Color::Black)
            .to_move(Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::A2.as_index());

        assert!(move_generator.moves.len() == 4);
        assert!(move_generator.generated_move(
            Square::A2,
            Square::A1,
            Flag::PromoteTo(Piece::Queen)
        ));
        assert!(move_generator.generated_move(
            Square::A2,
            Square::A1,
            Flag::PromoteTo(Piece::Rook)
        ));
        assert!(move_generator.generated_move(
            Square::A2,
            Square::A1,
            Flag::PromoteTo(Piece::Bishop)
        ));
        assert!(move_generator.generated_move(
            Square::A2,
            Square::A1,
            Flag::PromoteTo(Piece::Knight)
        ));

        Ok(())
    }

    #[test]
    fn test_promotion_pawn_move_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .piece(Square::E7, Piece::Pawn, Color::White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::E7.as_index());

        assert_eq!(move_generator.moves.len(), 4);
        assert!(move_generator.generated_move(
            Square::E7,
            Square::E8,
            Flag::PromoteTo(Piece::Queen)
        ));
        assert!(move_generator.generated_move(
            Square::E7,
            Square::E8,
            Flag::PromoteTo(Piece::Rook)
        ));
        assert!(move_generator.generated_move(
            Square::E7,
            Square::E8,
            Flag::PromoteTo(Piece::Bishop)
        ));
        assert!(move_generator.generated_move(
            Square::E7,
            Square::E8,
            Flag::PromoteTo(Piece::Knight)
        ));

        Ok(())
    }

    #[test]
    fn test_promotion_pawn_move_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .piece(Square::E2, Piece::Pawn, Color::Black)
            .to_move(Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::E2.as_index());

        assert_eq!(move_generator.moves.len(), 4);
        assert!(move_generator.generated_move(
            Square::E2,
            Square::E1,
            Flag::PromoteTo(Piece::Queen)
        ));
        assert!(move_generator.generated_move(
            Square::E2,
            Square::E1,
            Flag::PromoteTo(Piece::Rook)
        ));
        assert!(move_generator.generated_move(
            Square::E2,
            Square::E1,
            Flag::PromoteTo(Piece::Bishop)
        ));
        assert!(move_generator.generated_move(
            Square::E2,
            Square::E1,
            Flag::PromoteTo(Piece::Knight)
        ));

        Ok(())
    }

    #[test]
    fn test_promotion_pawn_capture_with_promotion_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .piece(Square::E7, Piece::Pawn, Color::White)
            .piece(Square::E8, Piece::Knight, Color::Black)
            .piece(Square::D8, Piece::Queen, Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::E7.as_index());

        assert_eq!(move_generator.moves.len(), 4);
        assert!(move_generator.generated_move(
            Square::E7,
            Square::D8,
            Flag::PromoteTo(Piece::Queen)
        ));
        assert!(move_generator.generated_move(
            Square::E7,
            Square::D8,
            Flag::PromoteTo(Piece::Rook)
        ));
        assert!(move_generator.generated_move(
            Square::E7,
            Square::D8,
            Flag::PromoteTo(Piece::Bishop)
        ));
        assert!(move_generator.generated_move(
            Square::E7,
            Square::D8,
            Flag::PromoteTo(Piece::Knight)
        ));

        Ok(())
    }

    #[test]
    fn test_promotion_pawn_capture_with_promotion_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .piece(Square::E2, Piece::Pawn, Color::Black)
            .piece(Square::E1, Piece::Knight, Color::White)
            .piece(Square::D1, Piece::Queen, Color::White)
            .to_move(Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::E2.as_index());

        assert_eq!(move_generator.moves.len(), 4);
        assert!(move_generator.generated_move(
            Square::E2,
            Square::D1,
            Flag::PromoteTo(Piece::Queen)
        ));
        assert!(move_generator.generated_move(
            Square::E2,
            Square::D1,
            Flag::PromoteTo(Piece::Rook)
        ));
        assert!(move_generator.generated_move(
            Square::E2,
            Square::D1,
            Flag::PromoteTo(Piece::Bishop)
        ));
        assert!(move_generator.generated_move(
            Square::E2,
            Square::D1,
            Flag::PromoteTo(Piece::Knight)
        ));

        Ok(())
    }

    #[test]
    fn test_white_en_passant_capture_left_in_center() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::E2, Square::E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::G8, Square::F6, Flag::None))
            .make_move(Move::from_square(Square::E4, Square::E5, Flag::None))
            .make_move(Move::from_square(Square::D7, Square::D5, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::E5.as_index());

        assert!(move_generator.moves.len() == 3);
        assert!(move_generator.generated_move(Square::E5, Square::E6, Flag::None));
        assert!(move_generator.generated_move(Square::E5, Square::D6, Flag::EnPassantCapture));
        assert!(move_generator.generated_move(Square::E5, Square::F6, Flag::None));

        Ok(())
    }

    #[test]
    fn test_white_en_passant_capture_right_in_center() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::E2, Square::E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::B8, Square::C6, Flag::None))
            .make_move(Move::from_square(Square::E4, Square::E5, Flag::None))
            .make_move(Move::from_square(Square::F7, Square::F5, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::E5.as_index());

        assert!(move_generator.moves.len() == 2);
        assert!(move_generator.generated_move(Square::E5, Square::E6, Flag::None));
        assert!(move_generator.generated_move(Square::E5, Square::F6, Flag::EnPassantCapture));

        Ok(())
    }

    #[test]
    fn test_black_en_passant_capture_right_in_center() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::G1, Square::F3, Flag::None))
            .make_move(Move::from_square(Square::E7, Square::E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::H1, Square::H2, Flag::None))
            .make_move(Move::from_square(Square::E5, Square::E4, Flag::None))
            .make_move(Move::from_square(Square::D2, Square::D4, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::E4.as_index());

        assert!(move_generator.moves.len() == 3);
        assert!(move_generator.generated_move(Square::E4, Square::E3, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D3, Flag::EnPassantCapture));
        assert!(move_generator.generated_move(Square::E4, Square::F3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_black_en_passant_capture_left_in_center() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::G1, Square::H3, Flag::None))
            .make_move(Move::from_square(Square::E7, Square::E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::A2, Square::A4, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::E5, Square::E4, Flag::None))
            .make_move(Move::from_square(Square::F2, Square::F4, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::E4.as_index());

        assert!(move_generator.moves.len() == 2);
        assert!(move_generator.generated_move(Square::E4, Square::F3, Flag::EnPassantCapture));
        assert!(move_generator.generated_move(Square::E4, Square::E3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_white_en_passant_capture_right_on_a_file() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::A2, Square::A4, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::B8, Square::C6, Flag::None))
            .make_move(Move::from_square(Square::A4, Square::A5, Flag::None))
            .make_move(Move::from_square(Square::B7, Square::B5, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::A5.as_index());

        assert!(move_generator.moves.len() == 2);
        assert!(move_generator.generated_move(Square::A5, Square::A6, Flag::None));
        assert!(move_generator.generated_move(Square::A5, Square::B6, Flag::EnPassantCapture));

        Ok(())
    }

    #[test]
    fn test_white_en_passant_capture_left_on_h_file() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::H2, Square::H4, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::B8, Square::C6, Flag::None))
            .make_move(Move::from_square(Square::H4, Square::H5, Flag::None))
            .make_move(Move::from_square(Square::G7, Square::G5, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::H5.as_index());

        assert!(move_generator.moves.len() == 2);
        assert!(move_generator.generated_move(Square::H5, Square::H6, Flag::None));
        assert!(move_generator.generated_move(Square::H5, Square::G6, Flag::EnPassantCapture));

        Ok(())
    }

    #[test]
    fn test_black_en_passant_capture_left_on_a_file() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::E2, Square::E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::A7, Square::A5, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::E4, Square::E5, Flag::None))
            .make_move(Move::from_square(Square::A5, Square::A4, Flag::None))
            .make_move(Move::from_square(Square::B2, Square::B4, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::A4.as_index());

        assert!(move_generator.moves.len() == 2);
        assert!(move_generator.generated_move(Square::A4, Square::A3, Flag::None));
        assert!(move_generator.generated_move(Square::A4, Square::B3, Flag::EnPassantCapture));

        Ok(())
    }

    #[test]
    fn test_black_en_passant_capture_right_on_h_file() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(Square::E2, Square::E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::H7, Square::H5, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::E4, Square::E5, Flag::None))
            .make_move(Move::from_square(Square::H5, Square::H4, Flag::None))
            .make_move(Move::from_square(Square::G2, Square::G4, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(Square::H4.as_index());

        assert!(move_generator.moves.len() == 2);
        assert!(move_generator.generated_move(Square::H4, Square::H3, Flag::None));
        assert!(move_generator.generated_move(Square::H4, Square::G3, Flag::EnPassantCapture));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::E4, Piece::King, Color::White)
            .piece(Square::E8, Piece::King, Color::Black)
            .piece(Square::A2, Piece::Pawn, Color::White)
            .piece(Square::A7, Piece::Pawn, Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(Square::E4.as_index());

        assert!(move_generator.moves.len() == 8);
        assert!(move_generator.generated_move(Square::E4, Square::E5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::F4, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D4, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::E3, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::F5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::F3, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_with_blocking_same_color_pieces_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::E4, Piece::King, Color::White)
            .piece(Square::E8, Piece::King, Color::Black)
            .piece(Square::A2, Piece::Pawn, Color::White)
            .piece(Square::A7, Piece::Pawn, Color::Black)
            .piece(Square::E5, Piece::Knight, Color::White)
            .piece(Square::F3, Piece::Knight, Color::White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(Square::E4.as_index());

        assert!(move_generator.moves.len() == 6);
        assert!(move_generator.generated_move(Square::E4, Square::F4, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D4, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::E3, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::F5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_with_captures_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::E4, Piece::King, Color::White)
            .piece(Square::E8, Piece::King, Color::Black)
            .piece(Square::A2, Piece::Pawn, Color::White)
            .piece(Square::A7, Piece::Pawn, Color::Black)
            .piece(Square::E5, Piece::Knight, Color::Black)
            .piece(Square::F3, Piece::Knight, Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(Square::E4.as_index());

        assert!(move_generator.moves.len() == 8);
        assert!(move_generator.generated_move(Square::E4, Square::E5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::F4, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D4, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::E3, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::F5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::F3, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::E4, Piece::King, Color::Black)
            .piece(Square::E1, Piece::King, Color::White)
            .piece(Square::A2, Piece::Pawn, Color::White)
            .piece(Square::A7, Piece::Pawn, Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(Square::E4.as_index());

        assert!(move_generator.moves.len() == 8);
        assert!(move_generator.generated_move(Square::E4, Square::E5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::F4, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D4, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::E3, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::F5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::F3, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_with_blocking_same_color_pieces_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::E4, Piece::King, Color::Black)
            .piece(Square::E1, Piece::King, Color::White)
            .piece(Square::A2, Piece::Pawn, Color::White)
            .piece(Square::A7, Piece::Pawn, Color::Black)
            .piece(Square::E5, Piece::Knight, Color::Black)
            .piece(Square::F3, Piece::Knight, Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(Square::E4.as_index());

        assert!(move_generator.moves.len() == 6);
        assert!(move_generator.generated_move(Square::E4, Square::F4, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D4, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::E3, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::F5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_with_captures_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::E4, Piece::King, Color::Black)
            .piece(Square::E1, Piece::King, Color::White)
            .piece(Square::A2, Piece::Pawn, Color::White)
            .piece(Square::A7, Piece::Pawn, Color::Black)
            .piece(Square::E5, Piece::Knight, Color::White)
            .piece(Square::F3, Piece::Knight, Color::White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(Square::E4.as_index());

        assert!(move_generator.moves.len() == 8);
        assert!(move_generator.generated_move(Square::E4, Square::E5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::F4, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D4, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::E3, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::F5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::F3, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D5, Flag::None));
        assert!(move_generator.generated_move(Square::E4, Square::D3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_king_basic_movement_no_pacman_h_file_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .piece(Square::E2, Piece::Pawn, Color::White)
            .piece(Square::E7, Piece::Pawn, Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(Square::H1.as_index());

        assert!(move_generator.moves.len() == 3);
        assert!(move_generator.generated_move(Square::H1, Square::H2, Flag::None));
        assert!(move_generator.generated_move(Square::H1, Square::G1, Flag::None));
        assert!(move_generator.generated_move(Square::H1, Square::G2, Flag::None));

        Ok(())
    }

    #[test]
    fn test_king_basic_movement_no_pacman_a_file_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::A1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .piece(Square::E2, Piece::Pawn, Color::White)
            .piece(Square::E7, Piece::Pawn, Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(Square::A1.as_index());

        assert!(move_generator.moves.len() == 3);
        assert!(move_generator.generated_move(Square::A1, Square::A2, Flag::None));
        assert!(move_generator.generated_move(Square::A1, Square::B1, Flag::None));
        assert!(move_generator.generated_move(Square::A1, Square::B2, Flag::None));

        Ok(())
    }

    #[test]
    fn test_king_basic_movement_no_pacman_h_file_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H8, Piece::King, Color::Black)
            .piece(Square::E2, Piece::Pawn, Color::White)
            .piece(Square::E7, Piece::Pawn, Color::Black)
            .to_move(Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(Square::H8.as_index());

        assert!(move_generator.moves.len() == 3);
        assert!(move_generator.generated_move(Square::H8, Square::H7, Flag::None));
        assert!(move_generator.generated_move(Square::H8, Square::G8, Flag::None));
        assert!(move_generator.generated_move(Square::H8, Square::G7, Flag::None));

        Ok(())
    }

    #[test]
    fn test_king_basic_movement_no_pacman_a_file_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(Square::A1, Piece::King, Color::White)
            .piece(Square::A8, Piece::King, Color::Black)
            .piece(Square::E2, Piece::Pawn, Color::White)
            .piece(Square::E7, Piece::Pawn, Color::Black)
            .to_move(Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(Square::A8.as_index());

        assert!(move_generator.moves.len() == 3);
        assert!(move_generator.generated_move(Square::A8, Square::A7, Flag::None));
        assert!(move_generator.generated_move(Square::A8, Square::B8, Flag::None));
        assert!(move_generator.generated_move(Square::A8, Square::B7, Flag::None));

        Ok(())
    }
}
