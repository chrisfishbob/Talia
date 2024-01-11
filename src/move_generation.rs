use anyhow::{anyhow, bail, Result};
use core::fmt;

use crate::board::Board;
use crate::piece::{Color, Piece};
use crate::square::Square;

#[derive(Eq, PartialEq, Clone)]
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

    pub fn try_from_uci(
        algebraic_notation: &str,
        move_generator: &mut MoveGenerator,
    ) -> Result<Self> {
        let promotion_piece = match algebraic_notation.chars().nth(4) {
            Some('q') => Some(Piece::Queen),
            Some('r') => Some(Piece::Rook),
            Some('n') => Some(Piece::Knight),
            Some('b') => Some(Piece::Bishop),
            None => None,
            _ => bail!("Not a known promotion piece of q, r, n or b"),
        };

        let starting_square =
            Square::from_algebraic_notation(&algebraic_notation[0..2])?.as_index();
        let target_square = Square::from_algebraic_notation(&algebraic_notation[2..4])?.as_index();

        let moves = move_generator.generate_moves();

        match promotion_piece {
            None => moves
                .into_iter()
                .find(|mv| {
                    mv.starting_square == starting_square && mv.target_square == target_square
                })
                .ok_or(anyhow!("Not a legal move")),
            Some(promotion_piece) => moves
                .into_iter()
                .find(|mv| {
                    mv.starting_square == starting_square
                        && mv.target_square == target_square
                        && match mv.flag {
                            Flag::PromoteTo(piece) if piece == promotion_piece => true,
                            Flag::CaptureWithPromotion(_, piece) if piece == promotion_piece => {
                                true
                            }
                            _ => false,
                        }
                })
                .ok_or(anyhow!("Not a legal move")),
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
        if self.flag != Flag::None {
            write!(f, ", {:?}", self.flag)?;
        }
        Ok(())
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = format!(
            "{:?}{:?}",
            Square::from_index(self.starting_square),
            Square::from_index(self.target_square),
        )
        .to_lowercase();

        match self.flag {
            Flag::PromoteTo(piece) | Flag::CaptureWithPromotion(_, piece) => {
                // Color::Black to get lowercase
                output.push_str(piece.to_symbol(Color::Black).to_string().as_str())
            }
            _ => {}
        }
        write!(f, "{output}",)?;
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Flag {
    None,
    KingsideCastle,
    QueensideCastle,
    PawnDoublePush,
    EnPassantCapture,
    PromoteTo(Piece),
    Capture(Piece),
    // captured piece, promotion piece
    CaptureWithPromotion(Piece, Piece),
}

pub struct MoveGenerator {
    num_squares_to_edge: [[usize; 8]; 64],
    direction_offsets: [isize; 8],
    pub board: Board,
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
            board,
        }
    }

    pub fn generate_moves(&mut self) -> Vec<Move> {
        let mut legal_moves = Vec::new();
        let pseudo_legal_moves = self.generate_pseudo_legal_moves();
        let to_move = self.board.to_move;

        for mv in pseudo_legal_moves {
            // If castling path is not clear, can't castle
            if (mv.flag == Flag::KingsideCastle || mv.flag == Flag::QueensideCastle)
                && !self.is_castling_path_clear(&mv)
            {
                continue;
            }

            self.board.move_piece(&mv);

            let in_check_after_move = self.is_in_check(to_move);

            self.board.unmake_move(&mv).unwrap();

            if !in_check_after_move {
                legal_moves.push(mv);
            }
        }

        legal_moves
    }

    fn generate_pseudo_legal_moves(&mut self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();

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
                Piece::Queen | Piece::Rook | Piece::Bishop => {
                    self.generate_sliding_moves(&mut moves, square)
                }
                Piece::Knight => self.generate_knight_moves(&mut moves, square),
                Piece::Pawn => self.generate_pawn_moves(&mut moves, square),
                Piece::King => self.generate_king_moves(&mut moves, square),
            }
        }

        moves
    }

    fn generate_sliding_moves(&mut self, moves: &mut Vec<Move>, start_square: usize) {
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
                            let captured_piece = self.board.squares[target_square]
                                .expect("piece should not be None if color exists");
                            moves.push(Move::new(
                                start_square,
                                target_square,
                                Flag::Capture(captured_piece),
                            ));
                        }
                        // Blocked by friendly piece, cannot go on further.
                        break;
                    }
                    None => {
                        // No piece on the current square, keep generating moves
                        moves.push(Move::new(start_square, target_square, Flag::None));
                    }
                }
            }
        }
    }

    fn generate_knight_moves(&mut self, moves: &mut Vec<Move>, start_square: usize) {
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
                None => moves.push(Move::new(start_square, target_square, Flag::None)),
                Some(color) if color != self.board.to_move => {
                    let captured_piece = self.board.squares[target_square]
                        .expect("piece should not be None if color exists");
                    moves.push(Move::new(
                        start_square,
                        target_square,
                        Flag::Capture(captured_piece),
                    ))
                }
                _ => continue,
            }
        }
    }

    fn generate_pawn_moves(&mut self, moves: &mut Vec<Move>, start_square: usize) {
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
                moves.push(Move::new(start_square, target_one_up_index, Flag::None));
            } else {
                self.add_promotion_moves(moves, start_square, target_one_up_index, None);
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
                .board_state
                .en_passant_square
                .is_some_and(|index| index == target_square);

            if is_occupied_by_opponent_piece || can_capture_en_passant {
                let target_rank = target_square / 8;
                let is_promotion_move = target_rank == 0 || target_rank == 7;

                if is_promotion_move {
                    let captured_piece = self.board.squares[target_square];
                    self.add_promotion_moves(moves, start_square, target_square, captured_piece);
                } else if can_capture_en_passant {
                    moves.push(Move::new(start_square, target_square, Flag::EnPassantCapture));
                } else {
                    let captured_piece = self.board.squares[target_square]
                        .expect("piece should not be None if color exists");
                    moves.push(Move::new(
                        start_square,
                        target_square,
                        Flag::Capture(captured_piece),
                    ));
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
            moves.push(Move::new(
                start_square,
                target_two_up_index as usize,
                Flag::PawnDoublePush,
            ));
        }
    }

    fn generate_king_moves(&mut self, moves: &mut Vec<Move>, start_square: usize) {
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

            if self.board.colors[target_square].is_none() {
                moves.push(Move::new(start_square, target_square, Flag::None));
            } else if self.board.colors[target_square]
                .is_some_and(|color| color != self.board.colors[start_square].unwrap())
            {
                let captured_piece = self.board.squares[target_square]
                    .expect("piece should not be None if color exists");
                moves.push(Move::new(start_square, target_square, Flag::Capture(captured_piece)));
            }
        }

        // TODO: Refactor this
        match self.board.to_move {
            Color::White => {
                let kingside_castling_path_clear = self.board.squares[Square::F1.as_index()]
                    .is_none()
                    && self.board.squares[Square::G1.as_index()].is_none();
                if self.board.board_state.white_kingside_castling_priviledge
                    && kingside_castling_path_clear
                {
                    moves.push(Move::new(
                        start_square,
                        Square::G1.as_index(),
                        Flag::KingsideCastle,
                    ));
                }

                let queenside_castling_path_clear = self.board.squares[Square::D1.as_index()]
                    .is_none()
                    && self.board.squares[Square::C1.as_index()].is_none()
                    && self.board.squares[Square::B1.as_index()].is_none();
                if self.board.board_state.white_queenside_castling_priviledge
                    && queenside_castling_path_clear
                {
                    moves.push(Move::new(
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
                if self.board.board_state.black_kingside_castling_priviledge
                    && kingside_castling_path_clear
                {
                    moves.push(Move::new(
                        start_square,
                        Square::G8.as_index(),
                        Flag::KingsideCastle,
                    ));
                }

                let queenside_castling_path_clear = self.board.squares[Square::D8.as_index()]
                    .is_none()
                    && self.board.squares[Square::C8.as_index()].is_none()
                    && self.board.squares[Square::B8.as_index()].is_none();
                if self.board.board_state.black_queenside_castling_priviledge
                    && queenside_castling_path_clear
                {
                    moves.push(Move::new(
                        start_square,
                        Square::C8.as_index(),
                        Flag::QueensideCastle,
                    ))
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

    fn add_promotion_moves(
        &mut self,
        moves: &mut Vec<Move>,
        start: usize,
        target: usize,
        captured_piece: Option<Piece>,
    ) {
        match captured_piece {
            None => {
                moves.push(Move::new(start, target, Flag::PromoteTo(Piece::Queen)));
                moves.push(Move::new(start, target, Flag::PromoteTo(Piece::Rook)));
                moves.push(Move::new(start, target, Flag::PromoteTo(Piece::Bishop)));
                moves.push(Move::new(start, target, Flag::PromoteTo(Piece::Knight)));
            }
            Some(piece) => {
                moves.push(Move::new(
                    start,
                    target,
                    Flag::CaptureWithPromotion(piece, Piece::Queen),
                ));
                moves.push(Move::new(
                    start,
                    target,
                    Flag::CaptureWithPromotion(piece, Piece::Rook),
                ));
                moves.push(Move::new(
                    start,
                    target,
                    Flag::CaptureWithPromotion(piece, Piece::Bishop),
                ));
                moves.push(Move::new(
                    start,
                    target,
                    Flag::CaptureWithPromotion(piece, Piece::Knight),
                ));
            }
        }
    }

    pub fn is_pacman_move(start: usize, target: usize) -> bool {
        let starting_rank = start as isize / 8;
        let starting_file = start as isize % 8;
        let target_rank = target as isize / 8;
        let target_file = target as isize % 8;

        // Prevents pieces from teleporting from one side to another Pacman-style
        // Two ranks or columns is the most a non-sliding piece can legally move
        (target_rank - starting_rank).abs() > 2 || (target_file - starting_file).abs() > 2
    }

    pub fn is_in_check(&mut self, color_to_check: Color) -> bool {
        let king_square = (0..64)
            .find(|&square| {
                self.board.colors[square].is_some()
                    && self.board.squares[square].is_some()
                    && self.board.colors[square].unwrap() == color_to_check
                    && self.board.squares[square].unwrap() == Piece::King
            })
            .expect("could not find the king");

        let to_move = color_to_check.opposite_color();
        for square in 0..64 {
            if !self.board.colors[square].is_some_and(|color| color == to_move) {
                continue;
            }

            match self.board.squares[square].unwrap() {
                Piece::Pawn => {
                    let pawn_move_offsets = match self.board.to_move {
                        Color::White => [8, 16, 7, 9],
                        Color::Black => [-8, -16, -7, -9],
                    };

                    for capture_offset in &pawn_move_offsets[2..] {
                        let target_square = {
                            let tmp = square as isize + capture_offset;
                            if !(0..64).contains(&tmp) {
                                continue;
                            }
                            tmp as usize
                        };

                        if !Self::is_pacman_move(square, target_square)
                            && target_square == king_square
                        {
                            return true;
                        }
                    }
                }
                Piece::King => {
                    for offset in self.direction_offsets {
                        let target_square = {
                            let tmp = square as isize + offset;
                            if !(0..64).contains(&tmp) {
                                continue;
                            }
                            tmp as usize
                        };

                        if !Self::is_pacman_move(square, target_square)
                            && target_square == king_square
                        {
                            return true;
                        }
                    }
                }
                Piece::Knight => {
                    let knight_move_offsets = [-17, -15, -10, -6, 6, 10, 15, 17];

                    for offset in knight_move_offsets {
                        let target_square = {
                            let tmp = square as isize + offset;
                            if !(0..64).contains(&tmp) {
                                continue;
                            }
                            tmp as usize
                        };

                        if !Self::is_pacman_move(square, target_square)
                            && target_square == king_square
                        {
                            return true;
                        }
                    }
                }
                Piece::Bishop | Piece::Queen | Piece::Rook => {
                    let piece = self.board.squares[square]
                        .expect("should not be generating sliding moves from an empty square");

                    let start_direction_index = if piece == Piece::Bishop { 4 } else { 0 };
                    let end_direction_index = if piece == Piece::Rook { 4 } else { 8 };

                    for direction_index in start_direction_index..end_direction_index {
                        for n in 0..self.num_squares_to_edge[square][direction_index] {
                            let target_square = square as isize
                                + self.direction_offsets[direction_index] * (n as isize + 1);
                            let target_square = target_square as usize;
                            if target_square == king_square {
                                return true;
                            }

                            match self.board.colors[target_square] {
                                None => continue,
                                Some(_) => break,
                            }
                        }
                    }
                }
            }
        }

        false
    }

    fn calculate_opponent_attack_map(&mut self) -> [bool; 64] {
        let mut attack_map = [false; 64];
        let mut moves = Vec::new();
        let original_to_move = self.board.to_move;
        self.board.to_move = self.board.to_move.opposite_color();

        for square in 0..64 {
            if !self.board.colors[square].is_some_and(|color| color == self.board.to_move) {
                continue;
            }

            match self.board.squares[square].unwrap() {
                Piece::Pawn => {
                    let pawn_move_offsets = match self.board.to_move {
                        Color::White => [8, 16, 7, 9],
                        Color::Black => [-8, -16, -7, -9],
                    };

                    for capture_offset in &pawn_move_offsets[2..] {
                        let target_square = {
                            let tmp = square as isize + capture_offset;
                            if !(0..64).contains(&tmp) {
                                continue;
                            }
                            tmp as usize
                        };

                        if Self::is_pacman_move(square, target_square) {
                            continue;
                        }

                        attack_map[target_square] = true;
                    }
                }
                Piece::Queen | Piece::Bishop | Piece::Rook => {
                    self.generate_sliding_moves(&mut moves, square);
                }
                Piece::Knight => {
                    self.generate_knight_moves(&mut moves, square);
                }
                Piece::King => {
                    self.generate_king_moves(&mut moves, square);
                }
            }
        }

        for mv in moves {
            attack_map[mv.target_square] = true;
        }

        self.board.to_move = original_to_move;
        attack_map
    }

    fn is_castling_path_clear(&mut self, mv: &Move) -> bool {
        // TODO: Fix this outright war crime
        if mv.flag == Flag::KingsideCastle {
            let attacked_map = self.calculate_opponent_attack_map();

            if self.board.to_move == Color::White {
                if attacked_map[Square::E1.as_index()]
                    || attacked_map[Square::F1.as_index()]
                    || attacked_map[Square::G1.as_index()]
                {
                    return false;
                }
            } else if attacked_map[Square::E8.as_index()]
                || attacked_map[Square::F8.as_index()]
                || attacked_map[Square::G8.as_index()]
            {
                return false;
            }
        } else if mv.flag == Flag::QueensideCastle {
            let attacked_squares = self.calculate_opponent_attack_map();

            if self.board.to_move == Color::White {
                if attacked_squares[Square::E1.as_index()]
                    || attacked_squares[Square::D1.as_index()]
                    || attacked_squares[Square::C1.as_index()]
                {
                    return false;
                }
            } else if attacked_squares[Square::E8.as_index()]
                || attacked_squares[Square::D8.as_index()]
                || attacked_squares[Square::C8.as_index()]
            {
                return false;
            }
        }

        true
    }

    #[allow(unused)]
    fn can_kingside_castle(&self) -> bool {
        match self.board.to_move {
            Color::White => {
                self.board.board_state.white_kingside_castling_priviledge
                    && self.board.squares[Square::F1.as_index()].is_none()
                    && self.board.squares[Square::G1.as_index()].is_none()
            }
            Color::Black => {
                self.board.board_state.black_kingside_castling_priviledge
                    && self.board.squares[Square::F8.as_index()].is_none()
                    && self.board.squares[Square::G8.as_index()].is_none()
            }
        }
    }

    #[cfg(test)]
    fn perft_test(&mut self, depth: u32) -> u32 {
        if depth == 0 {
            return 1;
        }

        let mut num = 0;
        let moves = self.generate_moves();

        if depth == 1 {
            return moves.len() as u32;
        }

        for mv in moves.iter() {
            self.board.move_piece(mv);
            if !self.is_in_check(self.board.to_move.opposite_color()) {
                num += self.perft_test(depth - 1);
            }
            self.board.unmake_move(mv).unwrap();
        }

        num
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::board_builder::BoardBuilder;
    use crate::move_generation::{Flag, Move, MoveGenerator};
    use crate::piece::{
        Color::*,
        Piece::{self, *},
    };
    use crate::square::Square::{self, *};
    use anyhow::Result;

    #[test]
    fn test_move_uci_output() -> Result<()> {
        let mv = Move::from_square(Square::E4, Square::E5, Flag::None);
        let uci_output = format!("{mv}");

        dbg!(&uci_output);
        assert!(String::from("e4e5") == uci_output);

        Ok(())
    }

    #[test]
    fn test_move_uci_output_with_promotion() -> Result<()> {
        let mv = Move::from_square(Square::E7, Square::E8, Flag::PromoteTo(Piece::Queen));
        let uci_output = format!("{mv}");

        dbg!(&uci_output);
        assert!(String::from("e7e8q") == uci_output);

        Ok(())
    }

    #[test]
    fn test_move_uci_output_with_capture_with_promotion() -> Result<()> {
        let mv = Move::from_square(
            Square::E7,
            Square::F8,
            Flag::CaptureWithPromotion(Piece::Knight, Piece::Queen),
        );
        let uci_output = format!("{mv}");

        dbg!(&uci_output);
        assert!(String::from("e7f8q") == uci_output);

        Ok(())
    }

    #[test]
    fn test_move_generation_depth_1() -> Result<()> {
        let board = BoardBuilder::from_starting_position().try_into()?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(1);

        assert!(number_of_positions == 20);

        Ok(())
    }

    #[test]
    fn test_move_generation_depth_2() -> Result<()> {
        let board = BoardBuilder::from_starting_position().try_into()?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(2);

        assert!(number_of_positions == 400);

        Ok(())
    }

    #[test]
    fn test_move_generation_depth_3() -> Result<()> {
        let board = BoardBuilder::from_starting_position().try_into()?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(3);

        assert!(number_of_positions == 8902);

        Ok(())
    }

    #[test]
    fn test_move_generation_depth_4() -> Result<()> {
        let board = BoardBuilder::from_starting_position().try_into()?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(4);

        assert!(number_of_positions == 197281);

        Ok(())
    }

    #[ignore] // Too expensive. Run with cargo test -- --ignored
    #[test]
    fn test_move_generation_depth_5() -> Result<()> {
        let board = BoardBuilder::from_starting_position().try_into()?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(5);

        assert!(number_of_positions == 4865609);

        Ok(())
    }

    #[ignore] // Too expensive. Run with cargo test -- --ignored
    #[test]
    fn test_move_generation_depth_6() -> Result<()> {
        let board = BoardBuilder::from_starting_position().try_into()?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(6);

        assert!(number_of_positions == 119060324);

        Ok(())
    }

    #[test]
    fn test_move_generation_kiwipete_depth_1() -> Result<()> {
        let board = BoardBuilder::try_from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(1);

        assert!(number_of_positions == 48);

        Ok(())
    }

    #[test]
    fn test_move_generation_kiwipete_depth_2() -> Result<()> {
        let board = BoardBuilder::try_from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(2);

        assert!(number_of_positions == 2039);

        Ok(())
    }

    #[test]
    fn test_move_generation_kiwipete_depth_3() -> Result<()> {
        let board = BoardBuilder::try_from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(3);

        assert!(number_of_positions == 97862);

        Ok(())
    }

    #[test]
    fn test_move_generation_kiwipete_depth_4() -> Result<()> {
        let board = BoardBuilder::try_from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(4);

        assert!(number_of_positions == 4085603);

        Ok(())
    }

    #[test]
    fn test_move_generation_tricky_position_depth_1() -> Result<()> {
        let board = BoardBuilder::try_from_fen(
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        )?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(1);

        assert!(number_of_positions == 44);

        Ok(())
    }

    #[test]
    fn test_move_generation_tricky_position_depth_2() -> Result<()> {
        let board = BoardBuilder::try_from_fen(
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8  ",
        )?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(2);

        assert!(number_of_positions == 1486);

        Ok(())
    }

    #[test]
    fn test_move_generation_tricky_position_depth_3() -> Result<()> {
        let board = BoardBuilder::try_from_fen(
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8  ",
        )?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(3);

        assert!(number_of_positions == 62379);

        Ok(())
    }

    #[test]
    fn test_move_generation_tricky_position_depth_4() -> Result<()> {
        let board = BoardBuilder::try_from_fen(
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8  ",
        )?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(4);

        dbg!(number_of_positions);
        assert!(number_of_positions == 2103487);

        Ok(())
    }

    #[ignore] // Too expensive. Run with cargo test -- --ignored
    #[test]
    fn test_move_generation_tricky_position_depth_5() -> Result<()> {
        let board = BoardBuilder::try_from_fen(
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8  ",
        )?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(5);

        dbg!(number_of_positions);
        assert!(number_of_positions == 89941194);

        Ok(())
    }

    #[test]
    fn test_move_generation_edwards_perft_depth_1() -> Result<()> {
        let board = BoardBuilder::try_from_fen(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ",
        )?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(1);

        dbg!(number_of_positions);
        assert!(number_of_positions == 46);

        Ok(())
    }

    #[test]
    fn test_move_generation_edwards_perft_depth_2() -> Result<()> {
        let board = BoardBuilder::try_from_fen(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ",
        )?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(2);

        dbg!(number_of_positions);
        assert!(number_of_positions == 2079);

        Ok(())
    }

    #[test]
    fn test_move_generation_edwards_perft_depth_3() -> Result<()> {
        let board = BoardBuilder::try_from_fen(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ",
        )?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(3);

        dbg!(number_of_positions);
        assert!(number_of_positions == 89890);

        Ok(())
    }

    #[test]
    fn test_move_generation_edwards_perft_depth_4() -> Result<()> {
        let board = BoardBuilder::try_from_fen(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ",
        )?;
        let mut move_generator = MoveGenerator::new(board);
        let number_of_positions = move_generator.perft_test(4);

        dbg!(number_of_positions);
        assert!(number_of_positions == 3894594);

        Ok(())
    }

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
        let mut moves = Vec::new();
        move_generator.generate_sliding_moves(&mut moves, A1.as_index());
        move_generator.generate_sliding_moves(&mut moves, C1.as_index());
        move_generator.generate_sliding_moves(&mut moves, D1.as_index());
        move_generator.generate_sliding_moves(&mut moves, F1.as_index());
        move_generator.generate_sliding_moves(&mut moves, H1.as_index());
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn test_generate_sliding_moves_empty_black() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_sliding_moves(&mut moves, A8.as_index());
        move_generator.generate_sliding_moves(&mut moves, C8.as_index());
        move_generator.generate_sliding_moves(&mut moves, D8.as_index());
        move_generator.generate_sliding_moves(&mut moves, F8.as_index());
        move_generator.generate_sliding_moves(&mut moves, H8.as_index());
        assert_eq!(moves.len(), 0);
        Ok(())
    }

    #[test]
    fn test_generate_sliding_moves_from_e4_e5() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_sliding_moves(&mut moves, A1.as_index());
        move_generator.generate_sliding_moves(&mut moves, C1.as_index());
        move_generator.generate_sliding_moves(&mut moves, D1.as_index());
        move_generator.generate_sliding_moves(&mut moves, F1.as_index());
        move_generator.generate_sliding_moves(&mut moves, H1.as_index());

        assert!(moves.contains(&Move::from_square(D1, E2, Flag::None)));
        assert!(moves.contains(&Move::from_square(D1, F3, Flag::None)));
        assert!(moves.contains(&Move::from_square(D1, G4, Flag::None)));
        assert!(moves.contains(&Move::from_square(D1, H5, Flag::None)));
        assert!(moves.contains(&Move::from_square(F1, E2, Flag::None)));
        assert!(moves.contains(&Move::from_square(F1, D3, Flag::None)));
        assert!(moves.contains(&Move::from_square(F1, C4, Flag::None)));
        assert!(moves.contains(&Move::from_square(F1, B5, Flag::None)));
        assert!(moves.contains(&Move::from_square(F1, A6, Flag::None)));
        assert_eq!(moves.len(), 9);
        Ok(())
    }

    #[test]
    fn test_generate_sliding_moves_from_e4_e5_nf3() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::None))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_sliding_moves(&mut moves, A8.as_index());
        move_generator.generate_sliding_moves(&mut moves, C8.as_index());
        move_generator.generate_sliding_moves(&mut moves, D8.as_index());
        move_generator.generate_sliding_moves(&mut moves, F8.as_index());
        move_generator.generate_sliding_moves(&mut moves, H8.as_index());

        assert!(moves.contains(&Move::from_square(D8, E7, Flag::None)));
        assert!(moves.contains(&Move::from_square(D8, F6, Flag::None)));
        assert!(moves.contains(&Move::from_square(D8, G5, Flag::None)));
        assert!(moves.contains(&Move::from_square(D8, H4, Flag::None)));
        assert!(moves.contains(&Move::from_square(F8, E7, Flag::None)));
        assert!(moves.contains(&Move::from_square(F8, D6, Flag::None)));
        assert!(moves.contains(&Move::from_square(F8, C5, Flag::None)));
        assert!(moves.contains(&Move::from_square(F8, B4, Flag::None)));
        assert!(moves.contains(&Move::from_square(F8, A3, Flag::None)));
        assert_eq!(moves.len(), 9);

        Ok(())
    }

    #[test]
    fn test_generate_sliding_moves_from_e4_e5_nf3_nc6() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::None))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(B8, C6, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_sliding_moves(&mut moves, A1.as_index());
        move_generator.generate_sliding_moves(&mut moves, C1.as_index());
        move_generator.generate_sliding_moves(&mut moves, D1.as_index());
        move_generator.generate_sliding_moves(&mut moves, F1.as_index());
        move_generator.generate_sliding_moves(&mut moves, H1.as_index());

        assert!(moves.contains(&Move::from_square(D1, E2, Flag::None)));
        assert!(moves.contains(&Move::from_square(F1, E2, Flag::None)));
        assert!(moves.contains(&Move::from_square(F1, D3, Flag::None)));
        assert!(moves.contains(&Move::from_square(F1, C4, Flag::None)));
        assert!(moves.contains(&Move::from_square(F1, B5, Flag::None)));
        assert!(moves.contains(&Move::from_square(F1, A6, Flag::None)));
        assert!(moves.contains(&Move::from_square(H1, G1, Flag::None)));
        assert_eq!(moves.len(), 7);

        Ok(())
    }

    #[test]
    fn test_generate_sliding_moves_from_corner() -> Result<()> {
        let board = BoardBuilder::try_from_fen("Qr5k/r7/2N5/8/8/8/8/6K1 w - - 0 1")?;
        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_sliding_moves(&mut moves, A8.as_index());

        assert_eq!(moves.len(), 3);
        assert!(moves.contains(&Move::from_square(A8, A7, Flag::Capture(Rook))));
        assert!(moves.contains(&Move::from_square(A8, B8, Flag::Capture(Rook))));
        assert!(moves.contains(&Move::from_square(A8, B7, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_generate_knight_moves_starting_position() {
        let mut move_generator = MoveGenerator::default();
        let mut moves = Vec::new();
        move_generator.generate_knight_moves(&mut moves, B1.as_index());
        move_generator.generate_knight_moves(&mut moves, G1.as_index());

        assert_eq!(moves.len(), 4);
        assert!(moves.contains(&Move::from_square(B1, A3, Flag::None)));
        assert!(moves.contains(&Move::from_square(B1, A3, Flag::None)));
        assert!(moves.contains(&Move::from_square(B1, C3, Flag::None)));
        assert!(moves.contains(&Move::from_square(G1, F3, Flag::None)));
        assert!(moves.contains(&Move::from_square(G1, H3, Flag::None)));
    }

    #[test]
    fn test_generate_knight_moves_from_corner() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(A1, King, White)
            .piece(A1, King, White)
            .piece(B1, Rook, White)
            .piece(H1, Knight, White)
            .piece(H8, King, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_knight_moves(&mut moves, H1.as_index());

        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::from_square(H1, F2, Flag::None)));
        assert!(moves.contains(&Move::from_square(H1, G3, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_generate_knight_moves_from_near_corner() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(A1, King, White)
            .piece(A1, King, White)
            .piece(B1, Rook, White)
            .piece(G2, Knight, White)
            .piece(H8, King, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_knight_moves(&mut moves, G2.as_index());

        assert_eq!(moves.len(), 4);
        assert!(moves.contains(&Move::from_square(G2, E1, Flag::None)));
        assert!(moves.contains(&Move::from_square(G2, E3, Flag::None)));
        assert!(moves.contains(&Move::from_square(G2, F4, Flag::None)));
        assert!(moves.contains(&Move::from_square(G2, H4, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_generate_knight_moves_with_pieces_on_target_square() -> Result<()> {
        let board = BoardBuilder::try_from_fen("k7/3R1n2/2n3R1/4N3/2R3n1/3n1R2/8/KR6 w - - 0 1")?;
        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_knight_moves(&mut moves, E5.as_index());
        assert_eq!(moves.len(), 4);
        assert!(moves.contains(&Move::from_square(E5, C6, Flag::Capture(Knight))));
        assert!(moves.contains(&Move::from_square(E5, D3, Flag::Capture(Knight))));
        assert!(moves.contains(&Move::from_square(E5, G4, Flag::Capture(Knight))));
        assert!(moves.contains(&Move::from_square(E5, F7, Flag::Capture(Knight))));

        Ok(())
    }

    #[test]
    fn test_generate_pawn_moves_from_starting_position_white() {
        let mut move_generator = MoveGenerator::default();
        let mut moves = Vec::new();

        for square in 0..64 {
            if move_generator
                .board
                .is_piece_at_square(square, Pawn, move_generator.board.to_move)
            {
                move_generator.generate_pawn_moves(&mut moves, square);
            }
        }

        assert_eq!(moves.len(), 16);
        assert!(moves.contains(&Move::from_square(A2, A3, Flag::None)));
        assert!(moves.contains(&Move::from_square(A2, A4, Flag::PawnDoublePush)));
        assert!(moves.contains(&Move::from_square(B2, B3, Flag::None)));
        assert!(moves.contains(&Move::from_square(B2, B4, Flag::PawnDoublePush)));
        assert!(moves.contains(&Move::from_square(C2, C3, Flag::None)));
        assert!(moves.contains(&Move::from_square(C2, C4, Flag::PawnDoublePush)));
        assert!(moves.contains(&Move::from_square(D2, D3, Flag::None)));
        assert!(moves.contains(&Move::from_square(D2, D4, Flag::PawnDoublePush)));
        assert!(moves.contains(&Move::from_square(E2, E3, Flag::None)));
        assert!(moves.contains(&Move::from_square(E2, E4, Flag::PawnDoublePush)));
        assert!(moves.contains(&Move::from_square(F2, F3, Flag::None)));
        assert!(moves.contains(&Move::from_square(F2, F4, Flag::PawnDoublePush)));
        assert!(moves.contains(&Move::from_square(G2, G3, Flag::None)));
        assert!(moves.contains(&Move::from_square(G2, G4, Flag::PawnDoublePush)));
        assert!(moves.contains(&Move::from_square(H2, H3, Flag::None)));
        assert!(moves.contains(&Move::from_square(H2, H4, Flag::PawnDoublePush)));
    }

    #[test]
    fn test_generate_pawn_moves_from_starting_position_black() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        for square in 0..64 {
            if move_generator
                .board
                .is_piece_at_square(square, Pawn, move_generator.board.to_move)
            {
                move_generator.generate_pawn_moves(&mut moves, square);
            }
        }

        assert_eq!(moves.len(), 16);
        assert!(moves.contains(&Move::from_square(A7, A5, Flag::PawnDoublePush)));
        assert!(moves.contains(&Move::from_square(A7, A6, Flag::None)));
        assert!(moves.contains(&Move::from_square(B7, B5, Flag::PawnDoublePush)));
        assert!(moves.contains(&Move::from_square(B7, B6, Flag::None)));
        assert!(moves.contains(&Move::from_square(C7, C5, Flag::PawnDoublePush)));
        assert!(moves.contains(&Move::from_square(C7, C6, Flag::None)));
        assert!(moves.contains(&Move::from_square(D7, D5, Flag::PawnDoublePush)));
        assert!(moves.contains(&Move::from_square(D7, D6, Flag::None)));
        assert!(moves.contains(&Move::from_square(E7, E5, Flag::PawnDoublePush)));
        assert!(moves.contains(&Move::from_square(E7, E6, Flag::None)));
        assert!(moves.contains(&Move::from_square(F7, F5, Flag::PawnDoublePush)));
        assert!(moves.contains(&Move::from_square(F7, F6, Flag::None)));
        assert!(moves.contains(&Move::from_square(G7, G5, Flag::PawnDoublePush)));
        assert!(moves.contains(&Move::from_square(G7, G6, Flag::None)));
        assert!(moves.contains(&Move::from_square(H7, H5, Flag::PawnDoublePush)));
        assert!(moves.contains(&Move::from_square(H7, H6, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_pawn_move_with_piece_blocking_white() -> Result<()> {
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
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, F4.as_index());
        move_generator.generate_pawn_moves(&mut moves, C4.as_index());

        assert_eq!(moves.len(), 0);

        Ok(())
    }

    #[test]
    fn test_pawn_move_with_piece_blocking_black() -> Result<()> {
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
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, F5.as_index());
        move_generator.generate_pawn_moves(&mut moves, C5.as_index());

        assert_eq!(moves.len(), 0);

        Ok(())
    }

    #[test]
    fn test_pawn_with_second_rank_blocked_white() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(E2, Pawn, White)
            .piece(E4, Pawn, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_pawn_moves(&mut moves, E2.as_index());

        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&Move::from_square(E2, E3, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_pawn_with_second_rank_blocked_black() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(E7, Pawn, Black)
            .piece(E5, Pawn, White)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, E7.as_index());

        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&Move::from_square(E7, E6, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_pawn_both_captures_in_center_white() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(D5, Pawn, Black)
            .piece(E4, Pawn, White)
            .piece(E5, Pawn, White)
            .piece(F5, Pawn, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_pawn_moves(&mut moves, E4.as_index());

        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::from_square(E4, D5, Flag::Capture(Pawn))));
        assert!(moves.contains(&Move::from_square(E4, F5, Flag::Capture(Pawn))));

        Ok(())
    }

    #[test]
    fn test_pawn_both_captures_in_center_black() -> Result<()> {
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
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, E5.as_index());

        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::from_square(E5, F4, Flag::Capture(Pawn))));
        assert!(moves.contains(&Move::from_square(E5, D4, Flag::Capture(Pawn))));

        Ok(())
    }

    #[test]
    fn test_pawn_no_pacman_white() -> Result<()> {
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
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, H4.as_index());

        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::from_square(H4, G5, Flag::Capture(Pawn))));
        assert!(moves.contains(&Move::from_square(H4, H5, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_pawn_no_pacman_black() -> Result<()> {
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
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, A5.as_index());

        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::from_square(A5, B4, Flag::Capture(Pawn))));
        assert!(moves.contains(&Move::from_square(A5, A4, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_pawn_no_anti_pacman_white() -> Result<()> {
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
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, A3.as_index());

        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&Move::from_square(A3, A4, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_pawn_no_anti_pacman_black() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(H5, Pawn, Black)
            .piece(A5, Pawn, White)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, H5.as_index());

        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&Move::from_square(H5, H4, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_already_moved_pawn_white() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::None))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, E4.as_index());

        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&Move::from_square(E4, E5, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_already_moved_pawn_black() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(H2, H4, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::None))
            .make_move(Move::from_square(H4, H5, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, E5.as_index());

        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&Move::from_square(E5, E4, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_pawn_capture_index_no_overflow() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(H7, Pawn, White)
            .piece(E1, King, White)
            .piece(E8, King, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, H7.as_index());

        assert!(moves.len() == 4);
        assert!(moves.contains(&Move::from_square(H7, H8, Flag::PromoteTo(Queen))));
        assert!(moves.contains(&Move::from_square(H7, H8, Flag::PromoteTo(Rook))));
        assert!(moves.contains(&Move::from_square(H7, H8, Flag::PromoteTo(Bishop))));
        assert!(moves.contains(&Move::from_square(H7, H8, Flag::PromoteTo(Knight))));

        Ok(())
    }

    #[test]
    fn test_pawn_capture_index_no_underflow() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(A2, Pawn, Black)
            .piece(E1, King, White)
            .piece(E8, King, Black)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, A2.as_index());

        assert!(moves.len() == 4);
        assert!(moves.contains(&Move::from_square(A2, A1, Flag::PromoteTo(Queen))));
        assert!(moves.contains(&Move::from_square(A2, A1, Flag::PromoteTo(Rook))));
        assert!(moves.contains(&Move::from_square(A2, A1, Flag::PromoteTo(Bishop))));
        assert!(moves.contains(&Move::from_square(A2, A1, Flag::PromoteTo(Knight))));

        Ok(())
    }

    #[test]
    fn test_promotion_pawn_move_white() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(E7, Pawn, White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, E7.as_index());

        assert_eq!(moves.len(), 4);
        assert!(moves.contains(&Move::from_square(E7, E8, Flag::PromoteTo(Queen))));
        assert!(moves.contains(&Move::from_square(E7, E8, Flag::PromoteTo(Rook))));
        assert!(moves.contains(&Move::from_square(E7, E8, Flag::PromoteTo(Bishop))));
        assert!(moves.contains(&Move::from_square(E7, E8, Flag::PromoteTo(Knight))));

        Ok(())
    }

    #[test]
    fn test_promotion_pawn_move_black() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(E2, Pawn, Black)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, E2.as_index());

        assert_eq!(moves.len(), 4);
        assert!(moves.contains(&Move::from_square(E2, E1, Flag::PromoteTo(Queen))));
        assert!(moves.contains(&Move::from_square(E2, E1, Flag::PromoteTo(Rook))));
        assert!(moves.contains(&Move::from_square(E2, E1, Flag::PromoteTo(Bishop))));
        assert!(moves.contains(&Move::from_square(E2, E1, Flag::PromoteTo(Knight))));

        Ok(())
    }

    #[test]
    fn test_promotion_pawn_capture_with_promotion_white() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(E7, Pawn, White)
            .piece(E8, Knight, Black)
            .piece(D8, Queen, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, E7.as_index());

        assert_eq!(moves.len(), 4);
        assert!(moves.contains(&Move::from_square(
            E7,
            D8,
            Flag::CaptureWithPromotion(Queen, Queen)
        )));
        assert!(moves.contains(&Move::from_square(
            E7,
            D8,
            Flag::CaptureWithPromotion(Queen, Rook)
        )));
        assert!(moves.contains(&Move::from_square(
            E7,
            D8,
            Flag::CaptureWithPromotion(Queen, Bishop)
        )));
        assert!(moves.contains(&Move::from_square(
            E7,
            D8,
            Flag::CaptureWithPromotion(Queen, Knight)
        )));

        Ok(())
    }

    #[test]
    fn test_promotion_pawn_capture_with_promotion_black() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(E2, Pawn, Black)
            .piece(E1, Knight, White)
            .piece(D1, Queen, White)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_pawn_moves(&mut moves, E2.as_index());

        assert_eq!(moves.len(), 4);
        assert!(moves.contains(&Move::from_square(
            E2,
            D1,
            Flag::CaptureWithPromotion(Queen, Queen)
        )));
        assert!(moves.contains(&Move::from_square(
            E2,
            D1,
            Flag::CaptureWithPromotion(Queen, Rook)
        )));
        assert!(moves.contains(&Move::from_square(
            E2,
            D1,
            Flag::CaptureWithPromotion(Queen, Bishop)
        )));
        assert!(moves.contains(&Move::from_square(
            E2,
            D1,
            Flag::CaptureWithPromotion(Queen, Knight)
        )));

        Ok(())
    }

    #[test]
    fn test_white_en_passant_capture_left_in_center() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .make_move(Move::from_square(E4, E5, Flag::None))
            .make_move(Move::from_square(D7, D5, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, E5.as_index());

        assert!(moves.len() == 3);
        assert!(moves.contains(&Move::from_square(E5, E6, Flag::None)));
        assert!(moves.contains(&Move::from_square(E5, D6, Flag::EnPassantCapture)));
        assert!(moves.contains(&Move::from_square(E5, F6, Flag::Capture(Knight))));

        Ok(())
    }

    #[test]
    fn test_white_en_passant_capture_right_in_center() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(B8, C6, Flag::None))
            .make_move(Move::from_square(E4, E5, Flag::None))
            .make_move(Move::from_square(F7, F5, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, E5.as_index());

        assert!(moves.len() == 2);
        assert!(moves.contains(&Move::from_square(E5, E6, Flag::None)));
        assert!(moves.contains(&Move::from_square(E5, F6, Flag::EnPassantCapture)));

        Ok(())
    }

    #[test]
    fn test_black_en_passant_capture_right_in_center() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(H1, H2, Flag::None))
            .make_move(Move::from_square(E5, E4, Flag::None))
            .make_move(Move::from_square(D2, D4, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();

        move_generator.generate_pawn_moves(&mut moves, E4.as_index());

        assert!(moves.len() == 3);
        assert!(moves.contains(&Move::from_square(E4, E3, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D3, Flag::EnPassantCapture)));
        assert!(moves.contains(&Move::from_square(E4, F3, Flag::Capture(Knight))));

        Ok(())
    }

    #[test]
    fn test_black_en_passant_capture_left_in_center() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(G1, H3, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(A2, A4, Flag::PawnDoublePush))
            .make_move(Move::from_square(E5, E4, Flag::None))
            .make_move(Move::from_square(F2, F4, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_pawn_moves(&mut moves, E4.as_index());

        assert!(moves.len() == 2);
        assert!(moves.contains(&Move::from_square(E4, F3, Flag::EnPassantCapture)));
        assert!(moves.contains(&Move::from_square(E4, E3, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_white_en_passant_capture_right_on_a_file() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(A2, A4, Flag::PawnDoublePush))
            .make_move(Move::from_square(B8, C6, Flag::None))
            .make_move(Move::from_square(A4, A5, Flag::None))
            .make_move(Move::from_square(B7, B5, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_pawn_moves(&mut moves, A5.as_index());

        assert!(moves.len() == 2);
        assert!(moves.contains(&Move::from_square(A5, A6, Flag::None)));
        assert!(moves.contains(&Move::from_square(A5, B6, Flag::EnPassantCapture)));

        Ok(())
    }

    #[test]
    fn test_white_en_passant_capture_left_on_h_file() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(H2, H4, Flag::PawnDoublePush))
            .make_move(Move::from_square(B8, C6, Flag::None))
            .make_move(Move::from_square(H4, H5, Flag::None))
            .make_move(Move::from_square(G7, G5, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_pawn_moves(&mut moves, H5.as_index());

        assert!(moves.len() == 2);
        assert!(moves.contains(&Move::from_square(H5, H6, Flag::None)));
        assert!(moves.contains(&Move::from_square(H5, G6, Flag::EnPassantCapture)));

        Ok(())
    }

    #[test]
    fn test_black_en_passant_capture_left_on_a_file() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(A7, A5, Flag::PawnDoublePush))
            .make_move(Move::from_square(E4, E5, Flag::None))
            .make_move(Move::from_square(A5, A4, Flag::None))
            .make_move(Move::from_square(B2, B4, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_pawn_moves(&mut moves, A4.as_index());

        assert!(moves.len() == 2);
        assert!(moves.contains(&Move::from_square(A4, A3, Flag::None)));
        assert!(moves.contains(&Move::from_square(A4, B3, Flag::EnPassantCapture)));

        Ok(())
    }

    #[test]
    fn test_black_en_passant_capture_right_on_h_file() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(H7, H5, Flag::PawnDoublePush))
            .make_move(Move::from_square(E4, E5, Flag::None))
            .make_move(Move::from_square(H5, H4, Flag::None))
            .make_move(Move::from_square(G2, G4, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_pawn_moves(&mut moves, H4.as_index());

        assert!(moves.len() == 2);
        assert!(moves.contains(&Move::from_square(H4, H3, Flag::None)));
        assert!(moves.contains(&Move::from_square(H4, G3, Flag::EnPassantCapture)));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_white() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(E4, King, White)
            .piece(E8, King, Black)
            .piece(A2, Pawn, White)
            .piece(A7, Pawn, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_king_moves(&mut moves, E4.as_index());

        assert!(moves.len() == 8);
        assert!(moves.contains(&Move::from_square(E4, E5, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, F4, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D4, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, E3, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, F5, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, F3, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D5, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D3, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_with_blocking_same_color_pieces_white() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(E4, King, White)
            .piece(E8, King, Black)
            .piece(A2, Pawn, White)
            .piece(A7, Pawn, Black)
            .piece(E5, Knight, White)
            .piece(F3, Knight, White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_king_moves(&mut moves, E4.as_index());

        assert!(moves.len() == 6);
        assert!(moves.contains(&Move::from_square(E4, F4, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D4, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, E3, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, F5, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D5, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D3, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_with_captures_white() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(E4, King, White)
            .piece(E8, King, Black)
            .piece(A2, Pawn, White)
            .piece(A7, Pawn, Black)
            .piece(E5, Knight, Black)
            .piece(F3, Knight, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_king_moves(&mut moves, E4.as_index());

        assert!(moves.len() == 8);
        assert!(moves.contains(&Move::from_square(E4, E5, Flag::Capture(Knight))));
        assert!(moves.contains(&Move::from_square(E4, F3, Flag::Capture(Knight))));
        assert!(moves.contains(&Move::from_square(E4, F4, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D4, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, E3, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, F5, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D5, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D3, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_black() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(E4, King, Black)
            .piece(E1, King, White)
            .piece(A2, Pawn, White)
            .piece(A7, Pawn, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_king_moves(&mut moves, E4.as_index());

        assert!(moves.len() == 8);
        assert!(moves.contains(&Move::from_square(E4, E5, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, F4, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D4, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, E3, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, F5, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, F3, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D5, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D3, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_with_blocking_same_color_pieces_black() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(E4, King, Black)
            .piece(E1, King, White)
            .piece(A2, Pawn, White)
            .piece(A7, Pawn, Black)
            .piece(E5, Knight, Black)
            .piece(F3, Knight, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_king_moves(&mut moves, E4.as_index());

        assert!(moves.len() == 6);
        assert!(moves.contains(&Move::from_square(E4, F4, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D4, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, E3, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, F5, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D5, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D3, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_basic_king_movement_with_captures_black() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(E4, King, Black)
            .piece(E1, King, White)
            .piece(A2, Pawn, White)
            .piece(A7, Pawn, Black)
            .piece(E5, Knight, White)
            .piece(F3, Knight, White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_king_moves(&mut moves, E4.as_index());

        assert!(moves.len() == 8);
        assert!(moves.contains(&Move::from_square(E4, E5, Flag::Capture(Knight))));
        assert!(moves.contains(&Move::from_square(E4, F3, Flag::Capture(Knight))));
        assert!(moves.contains(&Move::from_square(E4, F4, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D4, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, E3, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, F5, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D5, Flag::None)));
        assert!(moves.contains(&Move::from_square(E4, D3, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_king_basic_movement_no_pacman_h_file_white() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(E2, Pawn, White)
            .piece(E7, Pawn, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_king_moves(&mut moves, H1.as_index());

        assert!(moves.len() == 3);
        assert!(moves.contains(&Move::from_square(H1, H2, Flag::None)));
        assert!(moves.contains(&Move::from_square(H1, G1, Flag::None)));
        assert!(moves.contains(&Move::from_square(H1, G2, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_king_basic_movement_no_pacman_a_file_white() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(A1, King, White)
            .piece(H8, King, Black)
            .piece(E2, Pawn, White)
            .piece(E7, Pawn, Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_king_moves(&mut moves, A1.as_index());

        assert!(moves.len() == 3);
        assert!(moves.contains(&Move::from_square(A1, A2, Flag::None)));
        assert!(moves.contains(&Move::from_square(A1, B1, Flag::None)));
        assert!(moves.contains(&Move::from_square(A1, B2, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_king_basic_movement_no_pacman_h_file_black() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(H1, King, White)
            .piece(H8, King, Black)
            .piece(E2, Pawn, White)
            .piece(E7, Pawn, Black)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_king_moves(&mut moves, H8.as_index());

        assert!(moves.len() == 3);
        assert!(moves.contains(&Move::from_square(H8, H7, Flag::None)));
        assert!(moves.contains(&Move::from_square(H8, G8, Flag::None)));
        assert!(moves.contains(&Move::from_square(H8, G7, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_king_basic_movement_no_pacman_a_file_black() -> Result<()> {
        let board = BoardBuilder::new()
            .piece(A1, King, White)
            .piece(A8, King, Black)
            .piece(E2, Pawn, White)
            .piece(E7, Pawn, Black)
            .to_move(Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_king_moves(&mut moves, A8.as_index());

        assert!(moves.len() == 3);
        assert!(moves.contains(&Move::from_square(A8, A7, Flag::None)));
        assert!(moves.contains(&Move::from_square(A8, B8, Flag::None)));
        assert!(moves.contains(&Move::from_square(A8, B7, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_can_kingside_castle_starting_position_white() -> Result<()> {
        let board = Board::starting_position();

        let move_generator = MoveGenerator::new(board);

        assert!(!move_generator.can_kingside_castle());

        Ok(())
    }

    #[test]
    fn test_can_kingside_castle_white_true() -> Result<()> {
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
    fn test_can_kingside_castle_blocked_white() -> Result<()> {
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
    fn test_can_kingside_castle_starting_position_black() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .try_into()?;

        let move_generator = MoveGenerator::new(board);

        assert!(!move_generator.can_kingside_castle());

        Ok(())
    }

    #[test]
    fn test_can_kingside_castle_black_true() -> Result<()> {
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
    fn test_can_kingside_castle_blocked_black() -> Result<()> {
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
    fn test_kingside_castle_white() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(E7, E6, Flag::PawnDoublePush))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .make_move(Move::from_square(F1, C4, Flag::None))
            .make_move(Move::from_square(F8, C5, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_king_moves(&mut moves, E1.as_index());

        assert!(moves.len() == 3);
        assert!(moves.contains(&Move::from_square(E1, E2, Flag::None)));
        assert!(moves.contains(&Move::from_square(E1, F1, Flag::None)));
        assert!(moves.contains(&Move::from_square(E1, G1, Flag::KingsideCastle)));

        Ok(())
    }

    #[test]
    fn test_queenside_castle_white() -> Result<()> {
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
        let mut moves = Vec::new();
        move_generator.generate_king_moves(&mut moves, E1.as_index());

        assert!(moves.len() == 2);
        assert!(moves.contains(&Move::from_square(E1, C1, Flag::QueensideCastle)));
        assert!(moves.contains(&Move::from_square(E1, D1, Flag::None)));

        Ok(())
    }

    #[test]
    fn test_kingside_castle_black() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(E7, E6, Flag::PawnDoublePush))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .make_move(Move::from_square(F1, C4, Flag::None))
            .make_move(Move::from_square(F8, C5, Flag::None))
            .make_move(Move::from_square(H2, H3, Flag::None))
            .try_into()?;

        dbg!(&board);
        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_king_moves(&mut moves, E8.as_index());

        assert!(moves.len() == 3);
        assert!(moves.contains(&Move::from_square(E8, E7, Flag::None)));
        assert!(moves.contains(&Move::from_square(E8, F8, Flag::None)));
        assert!(moves.contains(&Move::from_square(E8, G8, Flag::KingsideCastle)));

        Ok(())
    }

    #[test]
    fn test_queenside_castle_black() -> Result<()> {
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

        dbg!(&board);
        let mut move_generator = MoveGenerator::new(board);
        let mut moves = Vec::new();
        move_generator.generate_king_moves(&mut moves, E8.as_index());

        assert!(moves.len() == 2);
        assert!(moves.contains(&Move::from_square(E8, D8, Flag::None)));
        assert!(moves.contains(&Move::from_square(E8, C8, Flag::QueensideCastle)));

        Ok(())
    }

    #[test]
    fn test_generate_moves_starting_position_white() -> Result<()> {
        let board: Board = BoardBuilder::from_starting_position().try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let moves = move_generator.generate_moves();

        assert!(moves.len() == 20);

        Ok(())
    }

    #[test]
    fn test_generate_moves_starting_position_black() -> Result<()> {
        let board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let moves = move_generator.generate_moves();

        assert!(moves.len() == 20);

        Ok(())
    }

    #[test]
    fn test_calculate_opponent_attack_squares_from_white() -> Result<()> {
        let board = BoardBuilder::from_starting_position().try_into()?;
        let mut move_generator = MoveGenerator::new(board);
        let attacked_squares = move_generator.calculate_opponent_attack_map();

        let squares_attacked = attacked_squares
            .iter()
            .filter(|&&attacked| attacked)
            .count();

        assert!(squares_attacked == 8);
        Ok(())
    }

    #[test]
    fn test_calculate_opponent_attack_squares_from_black() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let attacked_squares = move_generator.calculate_opponent_attack_map();

        let squares_attacked = attacked_squares
            .iter()
            .filter(|&&attacked| attacked)
            .count();

        assert!(squares_attacked == 16);
        Ok(())
    }

    #[test]
    fn test_is_kingside_castling_path_clear_true_white() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(B8, C6, Flag::None))
            .make_move(Move::from_square(F1, C4, Flag::None))
            .make_move(Move::from_square(G8, C5, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        assert!(move_generator.is_castling_path_clear(&Move::from_square(
            E1,
            G1,
            Flag::KingsideCastle
        )));

        Ok(())
    }

    #[test]
    fn test_is_kingside_castling_path_clear_true_black() -> Result<()> {
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
        assert!(move_generator.is_castling_path_clear(&Move::from_square(
            E8,
            G8,
            Flag::KingsideCastle
        )));

        Ok(())
    }

    #[test]
    fn test_is_kingside_castling_path_clear_f1_attacked_white() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(F6, H5, Flag::None))
            .make_move(Move::from_square(F1, C4, Flag::None))
            .make_move(Move::from_square(H5, G3, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);

        assert!(!move_generator.is_castling_path_clear(&Move::from_square(
            E1,
            G1,
            Flag::KingsideCastle
        )));

        Ok(())
    }

    #[test]
    fn test_is_kingside_castling_path_clear_f8_attacked_black() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(F3, G5, Flag::None))
            .make_move(Move::from_square(F8, C5, Flag::None))
            .make_move(Move::from_square(G5, H7, Flag::None))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .make_move(Move::from_square(A2, A3, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);

        assert!(!move_generator.is_castling_path_clear(&Move::from_square(
            E8,
            G8,
            Flag::KingsideCastle
        )));

        Ok(())
    }

    #[test]
    fn test_is_kingside_castling_path_clear_king_in_check_white() -> Result<()> {
        let board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(F6, D5, Flag::None))
            .make_move(Move::from_square(F1, C4, Flag::None))
            .make_move(Move::from_square(D5, B4, Flag::None))
            .make_move(Move::from_square(H2, H3, Flag::None))
            .make_move(Move::from_square(B4, C2, Flag::None))
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);

        assert!(!move_generator.is_castling_path_clear(&Move::from_square(
            E1,
            G1,
            Flag::KingsideCastle
        )));

        Ok(())
    }
}
