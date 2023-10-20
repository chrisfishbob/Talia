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
    KingsideCastle,
    QueensideCastle,
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
        self.generate_pseudo_legal_moves()
    }

    fn generate_pseudo_legal_moves(&mut self) -> Vec<Move> {
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

            // TODO: Refactor this
            match self.board.to_move {
                Color::White => {
                    let kingside_castling_path_clear = self.board.squares[Square::F1.as_index()]
                        .is_none()
                        && self.board.squares[Square::G1.as_index()].is_none();
                    if self.board.white_kingside_castling_priviledge && kingside_castling_path_clear
                    {
                        self.moves.push(Move::new(
                            start_square,
                            Square::G1.as_index(),
                            Flag::KingsideCastle,
                        ));
                    }

                    let queenside_castling_path_clear = self.board.squares[Square::D1.as_index()]
                        .is_none()
                        && self.board.squares[Square::C1.as_index()].is_none();
                    if self.board.white_queenside_castling_priviledge
                        && queenside_castling_path_clear
                    {
                        self.moves.push(Move::new(
                            start_square,
                            Square::C1.as_index(),
                            Flag::QueensideCastle,
                        ))
                    }
                }
                Color::Black => { 
                    let kingside_castling_path_clear = self.board.squares[Square::F8.as_index()]
                        .is_none()
                        && self.board.squares[Square::G8.as_index()].is_none();
                    if self.board.white_kingside_castling_priviledge && kingside_castling_path_clear
                    {
                        self.moves.push(Move::new(
                            start_square,
                            Square::G8.as_index(),
                            Flag::KingsideCastle,
                        ));
                    }

                    let queenside_castling_path_clear = self.board.squares[Square::D8.as_index()]
                        .is_none()
                        && self.board.squares[Square::C8.as_index()].is_none();
                    if self.board.white_queenside_castling_priviledge
                        && queenside_castling_path_clear
                    {
                        self.moves.push(Move::new(
                            start_square,
                            Square::C8.as_index(),
                            Flag::QueensideCastle,
                        ))
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

    #[allow(unused)]
    fn can_kingside_castle(&self) -> bool {
        match self.board.to_move {
            Color::White => {
                self.board.white_kingside_castling_priviledge
                    && self.board.squares[Square::F1.as_index()].is_none()
                    && self.board.squares[Square::G1.as_index()].is_none()
            }
            Color::Black => {
                self.board.black_kingside_castling_priviledge
                    && self.board.squares[Square::F8.as_index()].is_none()
                    && self.board.squares[Square::G8.as_index()].is_none()
            }
        }
    }

    #[cfg(test)]
    fn generated_move(&self, start: Square, target: Square, flag: Flag) -> bool {
        self.moves.contains(&Move::from_square(start, target, flag))
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::board_builder::BoardBuilder;
    use crate::errors::BoardError;
    use crate::move_generation::{Flag, Move, MoveGenerator};
    use crate::piece::{Color::*, Piece::*};
    use crate::square::Square::*;

    #[test]
    fn test_num_squares_to_edge() {
        let move_generator = MoveGenerator::default();
        // North
        assert_eq!(move_generator.num_squares_to_edge[A1.as_index()][0], 7);
        assert_eq!(move_generator.num_squares_to_edge[A4.as_index()][0], 4);
        assert_eq!(move_generator.num_squares_to_edge[A8.as_index()][0], 0);
        // South
        assert_eq!(move_generator.num_squares_to_edge[A1.as_index()][1], 0);
        assert_eq!(move_generator.num_squares_to_edge[A4.as_index()][1], 3);
        assert_eq!(move_generator.num_squares_to_edge[A8.as_index()][1], 7);
        // West
        assert_eq!(move_generator.num_squares_to_edge[A4.as_index()][2], 0);
        assert_eq!(move_generator.num_squares_to_edge[E4.as_index()][2], 4);
        assert_eq!(move_generator.num_squares_to_edge[H4.as_index()][2], 7);
        // East
        assert_eq!(move_generator.num_squares_to_edge[A4.as_index()][3], 7);
        assert_eq!(move_generator.num_squares_to_edge[E4.as_index()][3], 3);
        assert_eq!(move_generator.num_squares_to_edge[H4.as_index()][3], 0);
        // North West
        assert_eq!(move_generator.num_squares_to_edge[A1.as_index()][4], 0);
        assert_eq!(move_generator.num_squares_to_edge[E4.as_index()][4], 4);
        assert_eq!(move_generator.num_squares_to_edge[H1.as_index()][4], 7);
        // South East
        assert_eq!(move_generator.num_squares_to_edge[A1.as_index()][5], 0);
        assert_eq!(move_generator.num_squares_to_edge[A8.as_index()][5], 7);
        assert_eq!(move_generator.num_squares_to_edge[E4.as_index()][5], 3);
        // North East
        assert_eq!(move_generator.num_squares_to_edge[A1.as_index()][6], 7);
        assert_eq!(move_generator.num_squares_to_edge[E4.as_index()][6], 3);
        assert_eq!(move_generator.num_squares_to_edge[H4.as_index()][6], 0);
        // South West
        assert_eq!(move_generator.num_squares_to_edge[A1.as_index()][7], 0);
        assert_eq!(move_generator.num_squares_to_edge[E4.as_index()][7], 3);
        assert_eq!(move_generator.num_squares_to_edge[H8.as_index()][7], 7);
    }

    #[test]
    fn test_generate_sliding_moves_empty_white() {
        let mut move_generator = MoveGenerator::default();
        move_generator.generate_sliding_moves(A1.as_index());
        move_generator.generate_sliding_moves(C1.as_index());
        move_generator.generate_sliding_moves(D1.as_index());
        move_generator.generate_sliding_moves(F1.as_index());
        move_generator.generate_sliding_moves(H1.as_index());
        assert_eq!(move_generator.moves.len(), 0);
    }

    #[test]
    fn test_generate_sliding_moves_empty_black() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);

        move_generator.generate_sliding_moves(A8.as_index());
        move_generator.generate_sliding_moves(C8.as_index());
        move_generator.generate_sliding_moves(D8.as_index());
        move_generator.generate_sliding_moves(F8.as_index());
        move_generator.generate_sliding_moves(H8.as_index());
        assert_eq!(move_generator.moves.len(), 0);
        Ok(())
    }

    #[test]
    fn test_generate_sliding_moves_from_e4_e5() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);

        move_generator.generate_sliding_moves(A1.as_index());
        move_generator.generate_sliding_moves(C1.as_index());
        move_generator.generate_sliding_moves(D1.as_index());
        move_generator.generate_sliding_moves(F1.as_index());
        move_generator.generate_sliding_moves(H1.as_index());

        assert!(move_generator.generated_move(D1, E2, Flag::None));
        assert!(move_generator.generated_move(D1, F3, Flag::None));
        assert!(move_generator.generated_move(D1, G4, Flag::None));
        assert!(move_generator.generated_move(D1, H5, Flag::None));
        assert!(move_generator.generated_move(F1, E2, Flag::None));
        assert!(move_generator.generated_move(F1, D3, Flag::None));
        assert!(move_generator.generated_move(F1, C4, Flag::None));
        assert!(move_generator.generated_move(F1, B5, Flag::None));
        assert!(move_generator.generated_move(F1, A6, Flag::None));
        assert_eq!(move_generator.moves.len(), 9);
        Ok(())
    }

    #[test]
    fn test_generate_sliding_moves_from_e4_e5_nf3() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::None))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);

        move_generator.generate_sliding_moves(A8.as_index());
        move_generator.generate_sliding_moves(C8.as_index());
        move_generator.generate_sliding_moves(D8.as_index());
        move_generator.generate_sliding_moves(F8.as_index());
        move_generator.generate_sliding_moves(H8.as_index());

        assert!(move_generator.generated_move(D8, E7, Flag::None));
        assert!(move_generator.generated_move(D8, F6, Flag::None));
        assert!(move_generator.generated_move(D8, G5, Flag::None));
        assert!(move_generator.generated_move(D8, H4, Flag::None));
        assert!(move_generator.generated_move(F8, E7, Flag::None));
        assert!(move_generator.generated_move(F8, D6, Flag::None));
        assert!(move_generator.generated_move(F8, C5, Flag::None));
        assert!(move_generator.generated_move(F8, B4, Flag::None));
        assert!(move_generator.generated_move(F8, A3, Flag::None));
        assert_eq!(move_generator.moves.len(), 9);

        Ok(())
    }

    #[test]
    fn test_generate_sliding_moves_from_e4_e5_nf3_nc6() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::None))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(B8, C6, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);

        move_generator.generate_sliding_moves(A1.as_index());
        move_generator.generate_sliding_moves(C1.as_index());
        move_generator.generate_sliding_moves(D1.as_index());
        move_generator.generate_sliding_moves(F1.as_index());
        move_generator.generate_sliding_moves(H1.as_index());

        assert!(move_generator.generated_move(D1, E2, Flag::None));
        assert!(move_generator.generated_move(F1, E2, Flag::None));
        assert!(move_generator.generated_move(F1, D3, Flag::None));
        assert!(move_generator.generated_move(F1, C4, Flag::None));
        assert!(move_generator.generated_move(F1, B5, Flag::None));
        assert!(move_generator.generated_move(F1, A6, Flag::None));
        assert!(move_generator.generated_move(H1, G1, Flag::None));
        assert_eq!(move_generator.moves.len(), 7);

        Ok(())
    }

    #[test]
    fn test_generate_sliding_moves_from_corner() -> Result<(), BoardError> {
        let board = BoardBuilder::try_from_fen("Qr5k/r7/2N5/8/8/8/8/6K1 w - - 0 1")?;
        let mut move_generator = MoveGenerator::new(board);

        move_generator.generate_sliding_moves(A8.as_index());

        assert_eq!(move_generator.moves.len(), 3);
        assert!(move_generator.generated_move(A8, A7, Flag::None));
        assert!(move_generator.generated_move(A8, B8, Flag::None));
        assert!(move_generator.generated_move(A8, B7, Flag::None));

        Ok(())
    }

    #[test]
    fn test_generate_knight_moves_starting_position() {
        let mut move_generator = MoveGenerator::default();
        move_generator.generate_knight_moves(B1.as_index());
        move_generator.generate_knight_moves(G1.as_index());

        assert_eq!(move_generator.moves.len(), 4);
        assert!(move_generator.generated_move(B1, A3, Flag::None));
        assert!(move_generator.generated_move(B1, A3, Flag::None));
        assert!(move_generator.generated_move(B1, C3, Flag::None));
        assert!(move_generator.generated_move(G1, F3, Flag::None));
        assert!(move_generator.generated_move(G1, H3, Flag::None));
    }

    #[test]
    fn test_generate_knight_moves_from_corner() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(A1, King, White)
            .piece(A1, King, White)
            .piece(B1, Rook, White)
            .piece(H1, Knight, White)
            .piece(H8, King, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_knight_moves(H1.as_index());

        assert_eq!(move_generator.moves.len(), 2);
        assert!(move_generator.generated_move(H1, F2, Flag::None));
        assert!(move_generator.generated_move(H1, G3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_generate_knight_moves_from_near_corner() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(A1, King, White)
            .piece(A1, King, White)
            .piece(B1, Rook, White)
            .piece(G2, Knight, White)
            .piece(H8, King, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_knight_moves(G2.as_index());

        assert_eq!(move_generator.moves.len(), 4);
        assert!(move_generator.generated_move(G2, E1, Flag::None));
        assert!(move_generator.generated_move(G2, E3, Flag::None));
        assert!(move_generator.generated_move(G2, F4, Flag::None));
        assert!(move_generator.generated_move(G2, H4, Flag::None));

        Ok(())
    }

    #[test]
    fn test_generate_knight_moves_with_pieces_on_target_square() -> Result<(), BoardError> {
        let board = BoardBuilder::try_from_fen("k7/3R1n2/2n3R1/4N3/2R3n1/3n1R2/8/KR6 w - - 0 1")?;
        let mut move_generator = MoveGenerator::new(board);

        move_generator.generate_knight_moves(E5.as_index());
        assert_eq!(move_generator.moves.len(), 4);
        assert!(move_generator.generated_move(E5, C6, Flag::None));
        assert!(move_generator.generated_move(E5, D3, Flag::None));
        assert!(move_generator.generated_move(E5, G4, Flag::None));
        assert!(move_generator.generated_move(E5, F7, Flag::None));

        Ok(())
    }

    #[test]
    fn test_generate_pawn_moves_from_starting_position_white() {
        let mut move_generator = MoveGenerator::default();

        for square in 0..64 {
            if move_generator
                .board
                .is_piece_at_square(square, Pawn, move_generator.board.to_move)
            {
                move_generator.generate_pawn_moves(square);
            }
        }

        assert_eq!(move_generator.moves.len(), 16);
        assert!(move_generator.generated_move(A2, A3, Flag::None));
        assert!(move_generator.generated_move(A2, A4, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(B2, B3, Flag::None));
        assert!(move_generator.generated_move(B2, B4, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(C2, C3, Flag::None));
        assert!(move_generator.generated_move(C2, C4, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(D2, D3, Flag::None));
        assert!(move_generator.generated_move(D2, D4, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(E2, E3, Flag::None));
        assert!(move_generator.generated_move(E2, E4, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(F2, F3, Flag::None));
        assert!(move_generator.generated_move(F2, F4, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(G2, G3, Flag::None));
        assert!(move_generator.generated_move(G2, G4, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(H2, H3, Flag::None));
        assert!(move_generator.generated_move(H2, H4, Flag::PawnDoublePush));
    }

    #[test]
    fn test_generate_pawn_moves_from_starting_position_black() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);

        for square in 0..64 {
            if move_generator
                .board
                .is_piece_at_square(square, Pawn, move_generator.board.to_move)
            {
                move_generator.generate_pawn_moves(square);
            }
        }

        assert_eq!(move_generator.moves.len(), 16);
        assert!(move_generator.generated_move(A7, A5, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(A7, A6, Flag::None));
        assert!(move_generator.generated_move(B7, B5, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(B7, B6, Flag::None));
        assert!(move_generator.generated_move(C7, C5, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(C7, C6, Flag::None));
        assert!(move_generator.generated_move(D7, D5, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(D7, D6, Flag::None));
        assert!(move_generator.generated_move(E7, E5, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(E7, E6, Flag::None));
        assert!(move_generator.generated_move(F7, F5, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(F7, F6, Flag::None));
        assert!(move_generator.generated_move(G7, G5, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(G7, G6, Flag::None));
        assert!(move_generator.generated_move(H7, H5, Flag::PawnDoublePush));
        assert!(move_generator.generated_move(H7, H6, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_move_with_piece_blocking_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            // Tests that opposite color pieces block movement
            .piece(F4, Pawn, White)
            .piece(F5, Knight, Black)
            // Tests that same color pieces also block movement
            .piece(C4, Pawn, White)
            .piece(C5, Knight, White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(F4.as_index());
        move_generator.generate_pawn_moves(C4.as_index());

        assert_eq!(move_generator.moves.len(), 0);

        Ok(())
    }

    #[test]
    fn test_pawn_move_with_piece_blocking_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            // Tests that opposite color pieces block movement
            .piece(F5, Pawn, Black)
            .piece(F4, Knight, White)
            // Tests that same color pieces also block movement
            .piece(C5, Pawn, Black)
            .piece(C4, Knight, Black)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(F5.as_index());
        move_generator.generate_pawn_moves(C5.as_index());

        assert_eq!(move_generator.moves.len(), 0);

        Ok(())
    }

    #[test]
    fn test_pawn_with_second_rank_blocked_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(E2, Pawn, White)
            .piece(E4, Pawn, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(E2.as_index());

        assert_eq!(move_generator.moves.len(), 1);
        assert!(move_generator.generated_move(E2, E3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_with_second_rank_blocked_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(E7, Pawn, Black)
            .piece(E5, Pawn, White)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(E7.as_index());

        assert_eq!(move_generator.moves.len(), 1);
        assert!(move_generator.generated_move(E7, E6, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_both_captures_in_center_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(D5, Pawn, Black)
            .piece(E4, Pawn, White)
            .piece(E5, Pawn, White)
            .piece(F5, Pawn, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(E4.as_index());

        assert_eq!(move_generator.moves.len(), 2);
        assert!(move_generator.generated_move(E4, D5, Flag::None));
        assert!(move_generator.generated_move(E4, F5, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_both_captures_in_center_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(D4, Pawn, White)
            .piece(E5, Pawn, Black)
            .piece(E4, Pawn, Black)
            .piece(F4, Pawn, White)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(E5.as_index());

        assert_eq!(move_generator.moves.len(), 2);
        assert!(move_generator.generated_move(E5, F4, Flag::None));
        assert!(move_generator.generated_move(E5, D4, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_no_pacman_white() -> Result<(), BoardError> {
        // If pacman behavior exists, a capture offset of 9 for a pawn at the
        // 7th file will result in a square in the 0th file to become the target
        // square.
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(H4, Pawn, White)
            .piece(G5, Pawn, Black)
            // If the pacman behavior exists, the A6 pawn would be a target square
            .piece(A6, Pawn, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(H4.as_index());

        assert_eq!(move_generator.moves.len(), 2);
        assert!(move_generator.generated_move(H4, G5, Flag::None));
        assert!(move_generator.generated_move(H4, H5, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_no_pacman_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(A5, Pawn, Black)
            .piece(B4, Pawn, White)
            // If anti-pacman behavior exists, the H3 pawn would be a target square
            .piece(H3, Pawn, White)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(A5.as_index());

        assert_eq!(move_generator.moves.len(), 2);
        assert!(move_generator.generated_move(A5, B4, Flag::None));
        assert!(move_generator.generated_move(A5, A4, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_no_anti_pacman_white() -> Result<(), BoardError> {
        // If anti-pacman behavior exists, a capture offset for a pawn at the 0th
        // file will result in the square on the 8th file on the same rank to become
        // the target square.
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(A3, Pawn, White)
            // If the pacman behavior exists, the H3 pawn would be a target square
            .piece(H3, Pawn, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(A3.as_index());

        assert_eq!(move_generator.moves.len(), 1);
        assert!(move_generator.generated_move(A3, A4, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_no_anti_pacman_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(H5, Pawn, Black)
            .piece(A5, Pawn, White)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(H5.as_index());

        assert_eq!(move_generator.moves.len(), 1);
        assert!(move_generator.generated_move(H5, H4, Flag::None));

        Ok(())
    }

    #[test]
    fn test_already_moved_pawn_white() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::None))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(E4.as_index());

        assert_eq!(move_generator.moves.len(), 1);
        assert!(move_generator.generated_move(E4, E5, Flag::None));

        Ok(())
    }

    #[test]
    fn test_already_moved_pawn_black() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(H2, H4, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::None))
            .make_move(Move::from_square(H4, H5, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(E5.as_index());

        assert_eq!(move_generator.moves.len(), 1);
        assert!(move_generator.generated_move(E5, E4, Flag::None));

        Ok(())
    }

    #[test]
    fn test_pawn_capture_index_no_overflow() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(H7, Pawn, White)
            .piece(E1, King, White)
            .piece(E8, King, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(H7.as_index());

        assert!(move_generator.moves.len() == 4);
        assert!(move_generator.generated_move(H7, H8, Flag::PromoteTo(Queen)));
        assert!(move_generator.generated_move(H7, H8, Flag::PromoteTo(Rook)));
        assert!(move_generator.generated_move(H7, H8, Flag::PromoteTo(Bishop)));
        assert!(move_generator.generated_move(H7, H8, Flag::PromoteTo(Knight)));

        Ok(())
    }

    #[test]
    fn test_pawn_capture_index_no_underflow() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(A2, Pawn, Black)
            .piece(E1, King, White)
            .piece(E8, King, Black)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(A2.as_index());

        assert!(move_generator.moves.len() == 4);
        assert!(move_generator.generated_move(A2, A1, Flag::PromoteTo(Queen)));
        assert!(move_generator.generated_move(A2, A1, Flag::PromoteTo(Rook)));
        assert!(move_generator.generated_move(A2, A1, Flag::PromoteTo(Bishop)));
        assert!(move_generator.generated_move(A2, A1, Flag::PromoteTo(Knight)));

        Ok(())
    }

    #[test]
    fn test_promotion_pawn_move_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(E7, Pawn, White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(E7.as_index());

        assert_eq!(move_generator.moves.len(), 4);
        assert!(move_generator.generated_move(E7, E8, Flag::PromoteTo(Queen)));
        assert!(move_generator.generated_move(E7, E8, Flag::PromoteTo(Rook)));
        assert!(move_generator.generated_move(E7, E8, Flag::PromoteTo(Bishop)));
        assert!(move_generator.generated_move(E7, E8, Flag::PromoteTo(Knight)));

        Ok(())
    }

    #[test]
    fn test_promotion_pawn_move_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(E2, Pawn, Black)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(E2.as_index());

        assert_eq!(move_generator.moves.len(), 4);
        assert!(move_generator.generated_move(E2, E1, Flag::PromoteTo(Queen)));
        assert!(move_generator.generated_move(E2, E1, Flag::PromoteTo(Rook)));
        assert!(move_generator.generated_move(E2, E1, Flag::PromoteTo(Bishop)));
        assert!(move_generator.generated_move(E2, E1, Flag::PromoteTo(Knight)));

        Ok(())
    }

    #[test]
    fn test_promotion_pawn_capture_with_promotion_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(E7, Pawn, White)
            .piece(E8, Knight, Black)
            .piece(D8, Queen, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(E7.as_index());

        assert_eq!(move_generator.moves.len(), 4);
        assert!(move_generator.generated_move(E7, D8, Flag::PromoteTo(Queen)));
        assert!(move_generator.generated_move(E7, D8, Flag::PromoteTo(Rook)));
        assert!(move_generator.generated_move(E7, D8, Flag::PromoteTo(Bishop)));
        assert!(move_generator.generated_move(E7, D8, Flag::PromoteTo(Knight)));

        Ok(())
    }

    #[test]
    fn test_promotion_pawn_capture_with_promotion_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(E2, Pawn, Black)
            .piece(E1, Knight, White)
            .piece(D1, Queen, White)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(E2.as_index());

        assert_eq!(move_generator.moves.len(), 4);
        assert!(move_generator.generated_move(E2, D1, Flag::PromoteTo(Queen)));
        assert!(move_generator.generated_move(E2, D1, Flag::PromoteTo(Rook)));
        assert!(move_generator.generated_move(E2, D1, Flag::PromoteTo(Bishop)));
        assert!(move_generator.generated_move(E2, D1, Flag::PromoteTo(Knight)));

        Ok(())
    }

    #[test]
    fn test_white_en_passant_capture_left_in_center() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .make_move(Move::from_square(E4, E5, Flag::None))
            .make_move(Move::from_square(D7, D5, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(E5.as_index());

        assert!(move_generator.moves.len() == 3);
        assert!(move_generator.generated_move(E5, E6, Flag::None));
        assert!(move_generator.generated_move(E5, D6, Flag::EnPassantCapture));
        assert!(move_generator.generated_move(E5, F6, Flag::None));

        Ok(())
    }

    #[test]
    fn test_white_en_passant_capture_right_in_center() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(B8, C6, Flag::None))
            .make_move(Move::from_square(E4, E5, Flag::None))
            .make_move(Move::from_square(F7, F5, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(E5.as_index());

        assert!(move_generator.moves.len() == 2);
        assert!(move_generator.generated_move(E5, E6, Flag::None));
        assert!(move_generator.generated_move(E5, F6, Flag::EnPassantCapture));

        Ok(())
    }

    #[test]
    fn test_black_en_passant_capture_right_in_center() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(H1, H2, Flag::None))
            .make_move(Move::from_square(E5, E4, Flag::None))
            .make_move(Move::from_square(D2, D4, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(E4.as_index());

        assert!(move_generator.moves.len() == 3);
        assert!(move_generator.generated_move(E4, E3, Flag::None));
        assert!(move_generator.generated_move(E4, D3, Flag::EnPassantCapture));
        assert!(move_generator.generated_move(E4, F3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_black_en_passant_capture_left_in_center() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(G1, H3, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(A2, A4, Flag::PawnDoublePush))
            .make_move(Move::from_square(E5, E4, Flag::None))
            .make_move(Move::from_square(F2, F4, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(E4.as_index());

        assert!(move_generator.moves.len() == 2);
        assert!(move_generator.generated_move(E4, F3, Flag::EnPassantCapture));
        assert!(move_generator.generated_move(E4, E3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_white_en_passant_capture_right_on_a_file() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(A2, A4, Flag::PawnDoublePush))
            .make_move(Move::from_square(B8, C6, Flag::None))
            .make_move(Move::from_square(A4, A5, Flag::None))
            .make_move(Move::from_square(B7, B5, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(A5.as_index());

        assert!(move_generator.moves.len() == 2);
        assert!(move_generator.generated_move(A5, A6, Flag::None));
        assert!(move_generator.generated_move(A5, B6, Flag::EnPassantCapture));

        Ok(())
    }

    #[test]
    fn test_white_en_passant_capture_left_on_h_file() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(H2, H4, Flag::PawnDoublePush))
            .make_move(Move::from_square(B8, C6, Flag::None))
            .make_move(Move::from_square(H4, H5, Flag::None))
            .make_move(Move::from_square(G7, G5, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(H5.as_index());

        assert!(move_generator.moves.len() == 2);
        assert!(move_generator.generated_move(H5, H6, Flag::None));
        assert!(move_generator.generated_move(H5, G6, Flag::EnPassantCapture));

        Ok(())
    }

    #[test]
    fn test_black_en_passant_capture_left_on_a_file() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(A7, A5, Flag::PawnDoublePush))
            .make_move(Move::from_square(E4, E5, Flag::None))
            .make_move(Move::from_square(A5, A4, Flag::None))
            .make_move(Move::from_square(B2, B4, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(A4.as_index());

        assert!(move_generator.moves.len() == 2);
        assert!(move_generator.generated_move(A4, A3, Flag::None));
        assert!(move_generator.generated_move(A4, B3, Flag::EnPassantCapture));

        Ok(())
    }

    #[test]
    fn test_black_en_passant_capture_right_on_h_file() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(H7, H5, Flag::PawnDoublePush))
            .make_move(Move::from_square(E4, E5, Flag::None))
            .make_move(Move::from_square(H5, H4, Flag::None))
            .make_move(Move::from_square(G2, G4, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_pawn_moves(H4.as_index());

        assert!(move_generator.moves.len() == 2);
        assert!(move_generator.generated_move(H4, H3, Flag::None));
        assert!(move_generator.generated_move(H4, G3, Flag::EnPassantCapture));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(E4, King, White)
            .piece(E8, King, Black)
            .piece(A2, Pawn, White)
            .piece(A7, Pawn, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(E4.as_index());

        assert!(move_generator.moves.len() == 8);
        assert!(move_generator.generated_move(E4, E5, Flag::None));
        assert!(move_generator.generated_move(E4, F4, Flag::None));
        assert!(move_generator.generated_move(E4, D4, Flag::None));
        assert!(move_generator.generated_move(E4, E3, Flag::None));
        assert!(move_generator.generated_move(E4, F5, Flag::None));
        assert!(move_generator.generated_move(E4, F3, Flag::None));
        assert!(move_generator.generated_move(E4, D5, Flag::None));
        assert!(move_generator.generated_move(E4, D3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_with_blocking_same_color_pieces_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(E4, King, White)
            .piece(E8, King, Black)
            .piece(A2, Pawn, White)
            .piece(A7, Pawn, Black)
            .piece(E5, Knight, White)
            .piece(F3, Knight, White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(E4.as_index());

        assert!(move_generator.moves.len() == 6);
        assert!(move_generator.generated_move(E4, F4, Flag::None));
        assert!(move_generator.generated_move(E4, D4, Flag::None));
        assert!(move_generator.generated_move(E4, E3, Flag::None));
        assert!(move_generator.generated_move(E4, F5, Flag::None));
        assert!(move_generator.generated_move(E4, D5, Flag::None));
        assert!(move_generator.generated_move(E4, D3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_with_captures_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(E4, King, White)
            .piece(E8, King, Black)
            .piece(A2, Pawn, White)
            .piece(A7, Pawn, Black)
            .piece(E5, Knight, Black)
            .piece(F3, Knight, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(E4.as_index());

        assert!(move_generator.moves.len() == 8);
        assert!(move_generator.generated_move(E4, E5, Flag::None));
        assert!(move_generator.generated_move(E4, F4, Flag::None));
        assert!(move_generator.generated_move(E4, D4, Flag::None));
        assert!(move_generator.generated_move(E4, E3, Flag::None));
        assert!(move_generator.generated_move(E4, F5, Flag::None));
        assert!(move_generator.generated_move(E4, F3, Flag::None));
        assert!(move_generator.generated_move(E4, D5, Flag::None));
        assert!(move_generator.generated_move(E4, D3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(E4, King, Black)
            .piece(E1, King, White)
            .piece(A2, Pawn, White)
            .piece(A7, Pawn, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(E4.as_index());

        assert!(move_generator.moves.len() == 8);
        assert!(move_generator.generated_move(E4, E5, Flag::None));
        assert!(move_generator.generated_move(E4, F4, Flag::None));
        assert!(move_generator.generated_move(E4, D4, Flag::None));
        assert!(move_generator.generated_move(E4, E3, Flag::None));
        assert!(move_generator.generated_move(E4, F5, Flag::None));
        assert!(move_generator.generated_move(E4, F3, Flag::None));
        assert!(move_generator.generated_move(E4, D5, Flag::None));
        assert!(move_generator.generated_move(E4, D3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_with_blocking_same_color_pieces_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(E4, King, Black)
            .piece(E1, King, White)
            .piece(A2, Pawn, White)
            .piece(A7, Pawn, Black)
            .piece(E5, Knight, Black)
            .piece(F3, Knight, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(E4.as_index());

        assert!(move_generator.moves.len() == 6);
        assert!(move_generator.generated_move(E4, F4, Flag::None));
        assert!(move_generator.generated_move(E4, D4, Flag::None));
        assert!(move_generator.generated_move(E4, E3, Flag::None));
        assert!(move_generator.generated_move(E4, F5, Flag::None));
        assert!(move_generator.generated_move(E4, D5, Flag::None));
        assert!(move_generator.generated_move(E4, D3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_with_captures_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(E4, King, Black)
            .piece(E1, King, White)
            .piece(A2, Pawn, White)
            .piece(A7, Pawn, Black)
            .piece(E5, Knight, White)
            .piece(F3, Knight, White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(E4.as_index());

        assert!(move_generator.moves.len() == 8);
        assert!(move_generator.generated_move(E4, E5, Flag::None));
        assert!(move_generator.generated_move(E4, F4, Flag::None));
        assert!(move_generator.generated_move(E4, D4, Flag::None));
        assert!(move_generator.generated_move(E4, E3, Flag::None));
        assert!(move_generator.generated_move(E4, F5, Flag::None));
        assert!(move_generator.generated_move(E4, F3, Flag::None));
        assert!(move_generator.generated_move(E4, D5, Flag::None));
        assert!(move_generator.generated_move(E4, D3, Flag::None));

        Ok(())
    }

    #[test]
    fn test_king_basic_movement_no_pacman_h_file_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(E2, Pawn, White)
            .piece(E7, Pawn, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(H1.as_index());

        assert!(move_generator.moves.len() == 3);
        assert!(move_generator.generated_move(H1, H2, Flag::None));
        assert!(move_generator.generated_move(H1, G1, Flag::None));
        assert!(move_generator.generated_move(H1, G2, Flag::None));

        Ok(())
    }

    #[test]
    fn test_king_basic_movement_no_pacman_a_file_white() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(A1, King, White)
            .piece(H8, King, Black)
            .piece(E2, Pawn, White)
            .piece(E7, Pawn, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(A1.as_index());

        assert!(move_generator.moves.len() == 3);
        assert!(move_generator.generated_move(A1, A2, Flag::None));
        assert!(move_generator.generated_move(A1, B1, Flag::None));
        assert!(move_generator.generated_move(A1, B2, Flag::None));

        Ok(())
    }

    #[test]
    fn test_king_basic_movement_no_pacman_h_file_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(E2, Pawn, White)
            .piece(E7, Pawn, Black)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(H8.as_index());

        assert!(move_generator.moves.len() == 3);
        assert!(move_generator.generated_move(H8, H7, Flag::None));
        assert!(move_generator.generated_move(H8, G8, Flag::None));
        assert!(move_generator.generated_move(H8, G7, Flag::None));

        Ok(())
    }

    #[test]
    fn test_king_basic_movement_no_pacman_a_file_black() -> Result<(), BoardError> {
        let board = BoardBuilder::new()
            .piece(A1, King, White)
            .piece(A8, King, Black)
            .piece(E2, Pawn, White)
            .piece(E7, Pawn, Black)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(A8.as_index());

        assert!(move_generator.moves.len() == 3);
        assert!(move_generator.generated_move(A8, A7, Flag::None));
        assert!(move_generator.generated_move(A8, B8, Flag::None));
        assert!(move_generator.generated_move(A8, B7, Flag::None));

        Ok(())
    }

    #[test]
    fn test_can_kingside_castle_starting_position_white() -> Result<(), BoardError> {
        let board = Board::starting_position();

        let move_generator = MoveGenerator::new(board);

        assert!(!move_generator.can_kingside_castle());

        Ok(())
    }

    #[test]
    fn test_can_kingside_castle_white_true() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(B8, C6, Flag::None))
            .make_move(Move::from_square(F1, C4, Flag::None))
            .make_move(Move::from_square(G8, C5, Flag::None))
            .try_into()?;

        let move_generator = MoveGenerator::new(board);

        assert!(move_generator.can_kingside_castle());

        Ok(())
    }

    #[test]
    fn test_can_kingside_castle_blocked_white() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(B8, C6, Flag::None))
            .try_into()?;

        let move_generator = MoveGenerator::new(board);

        assert!(!move_generator.can_kingside_castle());

        Ok(())
    }

    #[test]
    fn test_can_kingside_castle_starting_position_black() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .try_into()?;

        let move_generator = MoveGenerator::new(board);

        assert!(!move_generator.can_kingside_castle());

        Ok(())
    }

    #[test]
    fn test_can_kingside_castle_black_true() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .make_move(Move::from_square(F1, C4, Flag::None))
            .make_move(Move::from_square(F8, C5, Flag::None))
            .make_move(Move::from_square(H2, H3, Flag::None))
            .try_into()?;

        let move_generator = MoveGenerator::new(board);

        assert!(move_generator.can_kingside_castle());

        Ok(())
    }

    #[test]
    fn test_can_kingside_castle_blocked_black() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .make_move(Move::from_square(F1, C4, Flag::None))
            .try_into()?;

        let move_generator = MoveGenerator::new(board);

        assert!(!move_generator.can_kingside_castle());

        Ok(())
    }

    #[test]
    fn test_kingside_castle_white() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(E7, E6, Flag::PawnDoublePush))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .make_move(Move::from_square(F1, C4, Flag::None))
            .make_move(Move::from_square(F8, C5, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(E1.as_index());

        assert!(move_generator.generated_move(E1, E2, Flag::None));
        assert!(move_generator.generated_move(E1, F1, Flag::None));
        assert!(move_generator.generated_move(E1, G1, Flag::KingsideCastle));

        Ok(())
    }

    #[test]
    fn test_queenside_castle_white() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(D2, D4, Flag::PawnDoublePush))
            .make_move(Move::from_square(D7, D6, Flag::PawnDoublePush))
            .make_move(Move::from_square(B1, C3, Flag::None))
            .make_move(Move::from_square(B8, C6, Flag::None))
            .make_move(Move::from_square(C1, F4, Flag::None))
            .make_move(Move::from_square(C8, F5, Flag::None))
            .make_move(Move::from_square(D1, D2, Flag::None))
            .make_move(Move::from_square(D8, D7, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(E1.as_index());

        assert!(move_generator.generated_move(E1, C1, Flag::QueensideCastle));
        assert!(move_generator.generated_move(E1, D1, Flag::None));

        Ok(())
    }

    #[test]
    fn test_kingside_castle_black() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(E7, E6, Flag::PawnDoublePush))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .make_move(Move::from_square(F1, C4, Flag::None))
            .make_move(Move::from_square(F8, C5, Flag::None))
            .make_move(Move::from_square(H2, H3, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(E8.as_index());

        assert!(move_generator.generated_move(E8, G8, Flag::KingsideCastle));

        Ok(())
    }

    #[test]
    fn test_queenside_castle_black() -> Result<(), BoardError> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(D2, D4, Flag::PawnDoublePush))
            .make_move(Move::from_square(D7, D6, Flag::PawnDoublePush))
            .make_move(Move::from_square(B1, C3, Flag::None))
            .make_move(Move::from_square(B8, C6, Flag::None))
            .make_move(Move::from_square(C1, F4, Flag::None))
            .make_move(Move::from_square(C8, F5, Flag::None))
            .make_move(Move::from_square(D1, D2, Flag::None))
            .make_move(Move::from_square(D8, D7, Flag::None))
            .make_move(Move::from_square(H2, H3, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        move_generator.generate_king_moves(E8.as_index());

        assert!(move_generator.generated_move(E8, C8, Flag::QueensideCastle));

        Ok(())
    }
}
