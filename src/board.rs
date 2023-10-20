use crate::board_builder::BoardBuilder;
use crate::move_generation::{Flag, Move};
use crate::piece::{Color, Piece};
use crate::square::Square;
use std::fmt;

#[derive(PartialEq, Eq)]
pub struct Board {
    pub squares: [Option<Piece>; 64],
    pub colors: [Option<Color>; 64],
    pub to_move: Color,
    pub white_kingside_castling_priviledge: bool,
    pub black_kingside_castling_priviledge: bool,
    pub white_queenside_castling_priviledge: bool,
    pub black_queenside_castling_priviledge: bool,
    pub en_passant_square: Option<usize>,
    pub half_move_clock: u32,
    pub full_move_number: u32,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            squares: [None; 64],
            colors: [None; 64],
            to_move: Color::White,
            white_kingside_castling_priviledge: false,
            white_queenside_castling_priviledge: false,
            black_kingside_castling_priviledge: false,
            black_queenside_castling_priviledge: false,
            en_passant_square: None,
            half_move_clock: 0,
            full_move_number: 1,
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut board_vec: Vec<Vec<char>> = Vec::new();

        for rank in (0..8).rev() {
            let mut row: Vec<char> = Vec::new();
            for file in 0..8 {
                let index = rank * 8 + file;
                let piece = self.squares[index];
                let color = self.colors[index];

                let character = match piece {
                    Some(piece) => format!(
                        "{}",
                        piece.to_symbol(color.expect("square occupied by piece must have color"))
                    )
                    .chars()
                    .next()
                    .unwrap(),
                    None => ' ',
                };
                row.push(character);
            }

            board_vec.push(row);
        }

        writeln!(f)?;
        for (i, rank) in board_vec.iter().enumerate() {
            let rank_num = 8 - i;
            writeln!(f, "{rank_num}  {:?}\n", rank)?;
        }

        writeln!(f, "     A    B    C    D    E    F    G    H\n")?;
        writeln!(f, "{:?} to move.", self.to_move)
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self)?;
        writeln!(f, "Fen: {}", self.to_fen())?;
        match &self.en_passant_square {
            Some(square) => writeln!(f, "en passant square: {:?}", square)?,
            None => writeln!(f, "no en passant square")?,
        };
        writeln!(
            f,
            "Can white king side castle: {}",
            self.white_kingside_castling_priviledge
        )?;
        writeln!(
            f,
            "Can white queen side castle: {}",
            self.white_kingside_castling_priviledge
        )?;
        writeln!(
            f,
            "Can black king side castle: {}",
            self.black_kingside_castling_priviledge
        )?;
        writeln!(
            f,
            "Can black queen side castle: {}",
            self.black_kingside_castling_priviledge
        )?;
        writeln!(f, "half move clock: {}", self.half_move_clock)?;
        writeln!(f, "full move number: {}", self.full_move_number)
    }
}

impl Board {
    pub fn starting_position() -> Self {
        BoardBuilder::try_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("failed to construct default board config")
    }
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut empty_squares = 0;
            for file in 0..8 {
                let index = rank * 8 + file;
                let piece = self.squares[index];
                let color = self.colors[index];
                match (piece, color) {
                    (Some(piece), Some(color)) => {
                        if empty_squares > 0 {
                            fen.push_str(&empty_squares.to_string());
                            empty_squares = 0;
                        }
                        fen.push(piece.to_symbol(color));
                    }
                    _ => empty_squares += 1,
                }
            }
            if empty_squares > 0 {
                fen.push_str(&empty_squares.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        fen.push(' ');
        match self.to_move {
            Color::White => fen.push('w'),
            Color::Black => fen.push('b'),
        };

        fen.push(' ');
        if self.white_kingside_castling_priviledge {
            fen.push('K');
        }
        if self.white_queenside_castling_priviledge {
            fen.push('Q');
        }
        if self.black_kingside_castling_priviledge {
            fen.push('k');
        }
        if self.black_queenside_castling_priviledge {
            fen.push('q');
        }
        if !(self.white_kingside_castling_priviledge
            || self.white_queenside_castling_priviledge
            || self.black_kingside_castling_priviledge
            || self.black_queenside_castling_priviledge)
        {
            fen.push('-')
        }

        // TODO: Should Talia support the newer FEN spec where en passant squares are only listed
        // if a opposite-color pawn is there to actually capture it?
        fen.push(' ');
        match self.en_passant_square {
            None => fen.push('-'),
            Some(square) => {
                let square_names = [
                    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", "a2", "b2", "c2", "d2", "e2",
                    "f2", "g2", "h2", "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", "a4", "b4",
                    "c4", "d4", "e4", "f4", "g4", "h4", "a5", "b5", "c5", "d5", "e5", "f5", "g5",
                    "h5", "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6", "a7", "b7", "c7", "d7",
                    "e7", "f7", "g7", "h7", "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
                ];
                fen.push_str(square_names[square]);
            }
        }

        fen.push(' ');
        fen.push_str(&self.half_move_clock.to_string());

        fen.push(' ');
        fen.push_str(&self.full_move_number.to_string());

        fen
    }

    // TODO: Should this return an error?
    // TODO: Handle castling
    pub fn move_piece(&mut self, mv: Move) {
        // With every move, the ability to en passant expires until a double pawn push
        let saved_en_passant_square = self.en_passant_square;
        self.en_passant_square = None;

        if self.is_fifty_move_rule_resetting_move(&mv) {
            self.half_move_clock = 0;
        } else {
            self.half_move_clock += 1;
        }

        match mv.flag {
            Flag::PawnDoublePush => {
                let pawn_one_move_offset = if self.to_move == Color::White { 8 } else { -8 };
                let en_passant_index = mv.starting_square as isize + pawn_one_move_offset;
                self.en_passant_square = Some(en_passant_index as usize);
            }
            Flag::EnPassantCapture => {
                let starting_piece_color =
                    self.colors[mv.starting_square].expect("cannot make a move from empty square");
                let en_passant_square =
                    saved_en_passant_square.expect("illegal en passant move played");
                let captured_pawn_index = if starting_piece_color == Color::White {
                    en_passant_square - 8
                } else {
                    en_passant_square + 8
                };

                self.squares[captured_pawn_index] = None;
                self.colors[captured_pawn_index] = None;
            }
            _ => (),
        }

        // If the kings moves, lose castling priviledge to both sides
        if self.squares[mv.starting_square].is_some_and(|piece| piece == Piece::King) {
            match self.to_move {
                Color::White => {
                    self.white_kingside_castling_priviledge = false;
                    self.white_queenside_castling_priviledge = false;
                }
                Color::Black => {
                    self.black_kingside_castling_priviledge = false;
                    self.black_queenside_castling_priviledge = false;
                }
            }
        }

        // The the rook moves, castling rights to that particular side is lost
        if self.squares[mv.starting_square].is_some_and(|piece| piece == Piece::Rook) {
            let is_from_starting_kingside_room_square = if self.to_move == Color::White {
                mv.starting_square == Square::H1.as_index()
            } else {
                mv.starting_square == Square::H8.as_index()
            };
            let is_from_starting_queenside_room_square = if self.to_move == Color::White {
                mv.starting_square == Square::A1.as_index()
            } else {
                mv.starting_square == Square::A8.as_index()
            };

            match self.to_move {
                Color::White => {
                    if is_from_starting_kingside_room_square {
                        self.white_kingside_castling_priviledge = false;
                    } else if is_from_starting_queenside_room_square {
                        self.white_queenside_castling_priviledge = false;
                    }
                }
                Color::Black => {
                    if is_from_starting_kingside_room_square {
                        self.black_kingside_castling_priviledge = false;
                    } else if is_from_starting_queenside_room_square {
                        self.black_queenside_castling_priviledge = false;
                    }
                }
            }
        }

        // The the rook is captured, castling rights to that particular side is lost
        if self.squares[mv.target_square].is_some_and(|piece| piece == Piece::Rook) {
            let is_to_starting_kingside_room_square = if self.to_move == Color::White {
                mv.target_square == Square::H8.as_index()
            } else {
                mv.target_square == Square::H1.as_index()
            };
            let is_to_starting_queenside_room_square = if self.to_move == Color::White {
                mv.target_square == Square::A8.as_index()
            } else {
                mv.target_square == Square::A1.as_index()
            };

            match self.to_move {
                Color::White => {
                    if is_to_starting_kingside_room_square {
                        self.black_kingside_castling_priviledge = false;
                    } else if is_to_starting_queenside_room_square {
                        self.black_queenside_castling_priviledge = false;
                    }
                }
                Color::Black => {
                    if is_to_starting_kingside_room_square {
                        self.white_kingside_castling_priviledge = false;
                    } else if is_to_starting_queenside_room_square {
                        self.white_queenside_castling_priviledge = false;
                    }
                }
            }
        }

        if let Flag::PromoteTo(piece) = mv.flag {
            self.squares[mv.target_square] = Some(piece);
        } else {
            self.squares[mv.target_square] = self.squares[mv.starting_square];
        }
        self.colors[mv.target_square] = self.colors[mv.starting_square];
        self.squares[mv.starting_square] = None;
        self.colors[mv.starting_square] = None;

        if self.to_move == Color::White {
            self.to_move = Color::Black;
        } else {
            self.to_move = Color::White;
            self.full_move_number += 1;
        }
    }

    pub fn put_piece(&mut self, square: usize, piece: Piece, color: Color) {
        self.squares[square] = Some(piece);
        self.colors[square] = Some(color);
    }

    pub fn is_piece_at_square(&self, index: usize, piece: Piece, color: Color) -> bool {
        match (self.squares[index], self.colors[index]) {
            (Some(s), Some(c)) => s == piece && c == color,
            _ => false,
        }
    }

    pub fn is_square_empty(&self, index: usize) -> bool {
        self.squares[index].is_none() && self.colors[index].is_none()
    }

    fn is_fifty_move_rule_resetting_move(&self, mv: &Move) -> bool {
        let is_pawn_move =
            self.squares[mv.starting_square].is_some_and(|piece| piece == Piece::Pawn);

        let is_non_en_passant_capture =
            self.colors[mv.target_square].is_some_and(|color| color != self.to_move);

        is_pawn_move || is_non_en_passant_capture
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::Board,
        board_builder::BoardBuilder,
        errors::BoardError,
        move_generation::{Flag, Move},
        piece::{Color::*, Piece::*},
        square::Square::*,
    };

    #[test]
    fn test_starting_position_board_config() {
        let board = Board::starting_position();
        assert!(board.is_piece_at_square(A1.as_index(), Rook, White));
        assert!(board.is_piece_at_square(B1.as_index(), Knight, White));
        assert!(board.is_piece_at_square(C1.as_index(), Bishop, White));
        assert!(board.is_piece_at_square(D1.as_index(), Queen, White));
        assert!(board.is_piece_at_square(E1.as_index(), King, White));
        assert!(board.is_piece_at_square(F1.as_index(), Bishop, White));
        assert!(board.is_piece_at_square(G1.as_index(), Knight, White));
        assert!(board.is_piece_at_square(H1.as_index(), Rook, White));

        for i in A2 as usize..=H2 as usize {
            assert_eq!(board.squares[i], Some(Pawn));
            assert_eq!(board.colors[i], Some(White))
        }

        for i in A3 as usize..=H6 as usize {
            assert_eq!(board.squares[i], None);
        }

        for i in A7 as usize..=H7 as usize {
            assert_eq!(board.squares[i], Some(Pawn));
            assert_eq!(board.colors[i], Some(Black))
        }

        assert!(board.is_piece_at_square(A8.as_index(), Rook, Black));
        assert!(board.is_piece_at_square(B8.as_index(), Knight, Black));
        assert!(board.is_piece_at_square(C8.as_index(), Bishop, Black));
        assert!(board.is_piece_at_square(D8.as_index(), Queen, Black));
        assert!(board.is_piece_at_square(E8.as_index(), King, Black));
        assert!(board.is_piece_at_square(F8.as_index(), Bishop, Black));
        assert!(board.is_piece_at_square(G8.as_index(), Knight, Black));
        assert!(board.is_piece_at_square(H8.as_index(), Rook, Black));

        assert_eq!(board.to_move, White);
        assert_eq!(board.en_passant_square, None);
        assert!(board.white_kingside_castling_priviledge);
        assert!(board.white_queenside_castling_priviledge);
        assert!(board.black_kingside_castling_priviledge);
        assert!(board.black_queenside_castling_priviledge);
        assert_eq!(board.half_move_clock, 0);
        assert_eq!(board.full_move_number, 1);
    }

    #[test]
    fn test_from_fen_empty_board() -> Result<(), BoardError> {
        let empty_board = Board::default();
        let empty_board_from_fen = BoardBuilder::try_from_fen("8/8/8/8/8/8/8/8 w - - 0 1")?;

        assert_eq!(empty_board, empty_board_from_fen);

        Ok(())
    }

    #[test]
    fn test_from_fen_sicilian_defense() -> Result<(), BoardError> {
        let starting_board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(C7, C5, Flag::PawnDoublePush))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .try_into()?;

        // Position after 1. e4, c5 => 2. Nf3
        let created_board = BoardBuilder::try_from_fen(
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
        )?;

        assert_eq!(starting_board, created_board);
        Ok(())
    }

    #[test]
    fn test_from_puzzle_fen() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::new()
            .piece(D1, Bishop, Black)
            .piece(A2, Pawn, White)
            .piece(B2, Pawn, White)
            .piece(F2, King, White)
            .piece(H2, Pawn, White)
            .piece(D4, Pawn, White)
            .piece(E4, Pawn, Black)
            .piece(A6, Pawn, Black)
            .piece(G6, Pawn, Black)
            .piece(B7, Pawn, Black)
            .piece(E7, Pawn, Black)
            .piece(C7, Rook, White)
            .piece(H7, Pawn, Black)
            .piece(F8, King, Black)
            .half_move_clock(1)
            .full_move_number(31)
            .try_into()?;

        let created_board =
            BoardBuilder::try_from_fen("5k2/1pR1p2p/p5p1/8/3Pp3/8/PP3K1P/3b4 w - - 1 31")?;

        assert_eq!(board, created_board);
        Ok(())
    }

    #[test]
    fn test_to_fen_empty_board() {
        let board = Board::default();
        assert_eq!(board.to_fen(), "8/8/8/8/8/8/8/8 w - - 0 1");
    }

    #[test]
    fn test_to_fen_starting_position() {
        let board = Board::starting_position();
        assert_eq!(
            board.to_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }

    #[test]
    fn test_to_fen_italian_game() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(B8, C6, Flag::None))
            .make_move(Move::from_square(F1, C4, Flag::None))
            .try_into()?;

        assert_eq!(
            board.to_fen(),
            "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3"
        );
        Ok(())
    }

    #[test]
    fn test_to_fen_advanced_caro_kann() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(C7, C6, Flag::None))
            .make_move(Move::from_square(D2, D4, Flag::PawnDoublePush))
            .make_move(Move::from_square(D7, D5, Flag::PawnDoublePush))
            .make_move(Move::from_square(E4, E5, Flag::None))
            .make_move(Move::from_square(C8, F5, Flag::None))
            .make_move(Move::from_square(F1, E2, Flag::None))
            .make_move(Move::from_square(E7, E6, Flag::None))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(C6, C5, Flag::None))
            .make_move(Move::from_square(C1, E3, Flag::None))
            .try_into()?;

        assert_eq!(
            board.to_fen(),
            "rn1qkbnr/pp3ppp/4p3/2ppPb2/3P4/4BN2/PPP1BPPP/RN1QK2R b KQkq - 1 6"
        );
        Ok(())
    }

    #[test]
    fn test_to_fen_marshall_attack() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(B8, C6, Flag::None))
            .make_move(Move::from_square(F1, B5, Flag::None))
            .make_move(Move::from_square(A7, A6, Flag::None))
            .make_move(Move::from_square(B5, A4, Flag::None))
            .make_move(Move::from_square(G8, F6, Flag::None))
            // TODO: Handle castling
            .make_move(Move::from_square(E1, G1, Flag::None))
            .make_move(Move::from_square(H1, F1, Flag::None))
            // end
            .make_move(Move::from_square(F8, E7, Flag::None))
            .make_move(Move::from_square(F1, E1, Flag::None))
            .make_move(Move::from_square(B7, B5, Flag::None))
            .make_move(Move::from_square(A4, B3, Flag::None))
            // TODO: Handle castling
            .make_move(Move::from_square(E8, G8, Flag::None))
            .make_move(Move::from_square(H8, F8, Flag::None))
            // end
            .make_move(Move::from_square(C2, C3, Flag::None))
            .make_move(Move::from_square(D7, D5, Flag::PawnDoublePush))
            // TODO: Remove this manual value set when move increment in implemented
            .half_move_clock(0)
            .full_move_number(9)
            // TODO: Remove this when castling is properly handled
            .can_kingside_castle(White, false)
            .can_kingside_castle(Black, false)
            .can_queenside_castle(White, false)
            .can_queenside_castle(Black, false)
            .try_into()?;

        assert_eq!(
            board.to_fen(),
            "r1bq1rk1/2p1bppp/p1n2n2/1p1pp3/4P3/1BP2N2/PP1P1PPP/RNBQR1K1 w - d6 0 9"
        );
        Ok(())
    }

    #[test]
    fn test_pawn_double_push_registers_en_passant_square() {
        let mut board = Board::starting_position();

        board.move_piece(Move::from_square(E2, E4, Flag::PawnDoublePush));
        assert!(board
            .en_passant_square
            .is_some_and(|square| square == E3.as_index()));

        board.move_piece(Move::from_square(E7, E5, Flag::PawnDoublePush));
        assert!(board
            .en_passant_square
            .is_some_and(|square| square == E6.as_index()));

        board.move_piece(Move::from_square(G1, F3, Flag::None));
        assert!(board.en_passant_square.is_none());
    }

    #[test]
    fn test_en_passant_capture_white() -> Result<(), BoardError> {
        let mut board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .make_move(Move::from_square(E4, E5, Flag::None))
            .make_move(Move::from_square(D7, D5, Flag::PawnDoublePush))
            .try_into()?;

        board.move_piece(Move::from_square(E5, D6, Flag::EnPassantCapture));

        assert!(board.is_square_empty(D5.as_index()));
        assert!(board.is_piece_at_square(D6.as_index(), Pawn, White));
        assert!(board.en_passant_square.is_none());

        Ok(())
    }

    #[test]
    fn test_en_passant_capture_black() -> Result<(), BoardError> {
        let mut board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(H1, H2, Flag::None))
            .make_move(Move::from_square(E5, E4, Flag::None))
            .make_move(Move::from_square(D2, D4, Flag::PawnDoublePush))
            .try_into()?;

        board.move_piece(Move::from_square(E4, D3, Flag::EnPassantCapture));

        assert!(board.is_square_empty(D4.as_index()));
        assert!(board.is_piece_at_square(D3.as_index(), Pawn, Black));
        assert!(board.en_passant_square.is_none());

        Ok(())
    }

    #[test]
    fn test_pawn_promotion_white() -> Result<(), BoardError> {
        let mut board: Board = BoardBuilder::new()
            .piece(H7, Pawn, White)
            .piece(E1, King, White)
            .piece(E8, King, Black)
            .try_into()?;

        board.move_piece(Move::from_square(H7, H8, Flag::PromoteTo(Queen)));

        assert!(board.is_square_empty(H7.as_index()));
        assert!(board.is_piece_at_square(H8.as_index(), Queen, White));

        Ok(())
    }

    #[test]
    fn test_pawn_promotion_black() -> Result<(), BoardError> {
        let mut board: Board = BoardBuilder::new()
            .piece(H2, Pawn, Black)
            .piece(E1, King, White)
            .piece(E8, King, Black)
            .to_move(Black)
            .try_into()?;

        board.move_piece(Move::from_square(H2, H1, Flag::PromoteTo(Queen)));

        assert!(board.is_square_empty(H2.as_index()));
        assert!(board.is_piece_at_square(H1.as_index(), Queen, Black));

        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_invalid_en_passant_move_panic() {
        let mut board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(D7, D6, Flag::None))
            .make_move(Move::from_square(E4, E5, Flag::None))
            .make_move(Move::from_square(D6, D5, Flag::None))
            .try_into()
            .unwrap();

        board.move_piece(Move::from_square(E5, D6, Flag::EnPassantCapture));
    }

    #[test]
    fn test_is_fifty_move_rule_resetting_move_pawn_push() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(B8, C6, Flag::None))
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .try_into()?;

        assert!(board.half_move_clock == 0);

        Ok(())
    }

    #[test]
    fn test_is_fifty_move_rule_resetting_move_capture() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .make_move(Move::from_square(B1, C3, Flag::None))
            .make_move(Move::from_square(F6, E4, Flag::None))
            .try_into()?;

        assert!(board.half_move_clock == 0);

        Ok(())
    }

    #[test]
    fn test_moving_king_loses_castling_priviledges_white() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(E1, E2, Flag::None))
            .try_into()?;

        assert!(!board.white_kingside_castling_priviledge);
        assert!(!board.white_queenside_castling_priviledge);

        Ok(())
    }

    #[test]
    fn test_moving_king_loses_castling_priviledges_black() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(E1, E2, Flag::None))
            .make_move(Move::from_square(E8, E7, Flag::None))
            .try_into()?;

        assert!(!board.black_kingside_castling_priviledge);
        assert!(!board.black_queenside_castling_priviledge);

        Ok(())
    }

    #[test]
    fn test_moving_h1_rook_loses_castling_priviledges_white() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(H1, G1, Flag::None))
            .try_into()?;

        assert!(!board.white_kingside_castling_priviledge);
        assert!(board.white_queenside_castling_priviledge);
        assert!(board.black_kingside_castling_priviledge);
        assert!(board.black_queenside_castling_priviledge);

        Ok(())
    }

    #[test]
    fn test_moving_a1_rook_loses_castling_priviledges_white() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(B1, C3, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(A1, B1, Flag::None))
            .try_into()?;

        assert!(board.white_kingside_castling_priviledge);
        assert!(!board.white_queenside_castling_priviledge);
        assert!(board.black_kingside_castling_priviledge);
        assert!(board.black_queenside_castling_priviledge);

        Ok(())
    }

    #[test]
    fn test_moving_h8_rook_loses_castling_priviledges_black() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(H8, G8, Flag::None))
            .try_into()?;

        assert!(board.white_kingside_castling_priviledge);
        assert!(board.white_queenside_castling_priviledge);
        assert!(!board.black_kingside_castling_priviledge);
        assert!(board.black_queenside_castling_priviledge);

        Ok(())
    }

    #[test]
    fn test_moving_a8_rook_loses_castling_priviledges_black() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(B8, F6, Flag::None))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(H8, G8, Flag::None))
            .try_into()?;

        assert!(board.white_kingside_castling_priviledge);
        assert!(board.white_queenside_castling_priviledge);
        assert!(!board.black_kingside_castling_priviledge);
        assert!(board.black_queenside_castling_priviledge);

        Ok(())
    }

    #[test]
    fn test_capturing_h1_rook_loses_castling_priviledges_white() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(G8, F6, Flag::None))
            .make_move(Move::from_square(E4, E5, Flag::None))
            .make_move(Move::from_square(F6, G4, Flag::None))
            .make_move(Move::from_square(D2, D4, Flag::PawnDoublePush))
            .make_move(Move::from_square(G4, F2, Flag::None))
            .make_move(Move::from_square(D4, D5, Flag::None))
            .make_move(Move::from_square(F2, H1, Flag::None))
            .try_into()?;

        assert!(!board.white_kingside_castling_priviledge);
        assert!(board.white_queenside_castling_priviledge);
        assert!(board.black_kingside_castling_priviledge);
        assert!(board.black_queenside_castling_priviledge);

        Ok(())
    }

    #[test]
    fn test_capturing_a1_rook_loses_castling_priviledges_white() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(E2, E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(G7, G6, Flag::None))
            .make_move(Move::from_square(G1, F3, Flag::None))
            .make_move(Move::from_square(F8, G7, Flag::None))
            .make_move(Move::from_square(B2, B3, Flag::None))
            .make_move(Move::from_square(G7, A1, Flag::None))
            .try_into()?;

        assert!(board.white_kingside_castling_priviledge);
        assert!(!board.white_queenside_castling_priviledge);
        assert!(board.black_kingside_castling_priviledge);
        assert!(board.black_queenside_castling_priviledge);

        Ok(())
    }

    #[test]
    fn test_capturing_h8_rook_loses_castling_priviledges_black() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(B2, B3, Flag::None))
            .make_move(Move::from_square(G7, G6, Flag::None))
            .make_move(Move::from_square(C1, B2, Flag::None))
            .make_move(Move::from_square(D7, D5, Flag::PawnDoublePush))
            .make_move(Move::from_square(B2, H8, Flag::None))
            .try_into()?;

        assert!(board.white_kingside_castling_priviledge);
        assert!(board.white_queenside_castling_priviledge);
        assert!(!board.black_kingside_castling_priviledge);
        assert!(board.black_queenside_castling_priviledge);

        Ok(())
    }

    #[test]
    fn test_capturing_a8_rook_loses_castling_priviledges_black() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::from_starting_position()
            .make_move(Move::from_square(G2, G3, Flag::None))
            .make_move(Move::from_square(B7, B6, Flag::None))
            .make_move(Move::from_square(F1, G2, Flag::None))
            .make_move(Move::from_square(E7, E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(G2, A8, Flag::None))
            .try_into()?;

        assert!(board.white_kingside_castling_priviledge);
        assert!(board.white_queenside_castling_priviledge);
        assert!(board.black_kingside_castling_priviledge);
        assert!(!board.black_queenside_castling_priviledge);

        Ok(())
    }
}
