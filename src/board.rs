use crate::board_builder::BoardBuilder;
use crate::move_generation::{Flag, Move};
use crate::piece::{Color, Piece};
use std::fmt;

#[derive(PartialEq, Eq)]
pub struct Board {
    pub squares: [Option<Piece>; 64],
    pub colors: [Option<Color>; 64],
    pub to_move: Color,
    pub can_white_king_side_castle: bool,
    pub can_black_king_side_castle: bool,
    pub can_white_queen_side_castle: bool,
    pub can_black_queen_side_castle: bool,
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
            can_white_king_side_castle: false,
            can_white_queen_side_castle: false,
            can_black_king_side_castle: false,
            can_black_queen_side_castle: false,
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
        writeln!(f, "Can white king side castle: {}", self.can_white_king_side_castle)?;
        writeln!(f, "Can white queen side castle: {}", self.can_white_king_side_castle)?;
        writeln!(f, "Can black king side castle: {}", self.can_black_king_side_castle)?;
        writeln!(f, "Can black queen side castle: {}", self.can_black_king_side_castle)?;
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
        if self.can_white_king_side_castle {
            fen.push('K');
        }
        if self.can_white_queen_side_castle {
            fen.push('Q');
        }
        if self.can_black_king_side_castle {
            fen.push('k');
        }
        if self.can_black_queen_side_castle {
            fen.push('q');
        }
        if !(self.can_white_king_side_castle
            || self.can_white_queen_side_castle
            || self.can_black_king_side_castle
            || self.can_black_queen_side_castle)
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
    // TODO: Handle en passant, castling, promotion, ...
    // TODO: Handle move increment
    pub fn move_piece(&mut self, mv: Move) {
        // With every move, the ability to en passant expires until a double pawn push
        self.en_passant_square = None;

        let starting_piece = self.squares[mv.starting_square];
        let starting_piece_color = self.colors[mv.starting_square];
        self.squares[mv.target_square] = starting_piece;
        self.colors[mv.target_square] = starting_piece_color;
        self.squares[mv.starting_square] = None;
        self.colors[mv.starting_square] = None;

        if mv.flag == Flag::PawnDoublePush {
            let pawn_one_move_offset = if self.to_move == Color::White { 8 } else { -8 };
            let en_passant_index = mv.starting_square as isize + pawn_one_move_offset;
            self.en_passant_square = Some(en_passant_index as usize)
        }

        if self.to_move == Color::White {
            self.to_move = Color::Black;
        } else {
            self.to_move = Color::White;
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
}

#[cfg(test)]
mod tests {
    use crate::{
        board::Board,
        board_builder::BoardBuilder,
        errors::BoardError,
        move_generation::{Flag, Move},
        piece::{Color, Piece},
        square::Square,
    };

    #[test]
    fn test_starting_position_board_config() {
        let board = Board::starting_position();
        assert!(board.is_piece_at_square(Square::A1.as_index(), Piece::Rook, Color::White));
        assert!(board.is_piece_at_square(Square::B1.as_index(), Piece::Knight, Color::White));
        assert!(board.is_piece_at_square(Square::C1.as_index(), Piece::Bishop, Color::White));
        assert!(board.is_piece_at_square(Square::D1.as_index(), Piece::Queen, Color::White));
        assert!(board.is_piece_at_square(Square::E1.as_index(), Piece::King, Color::White));
        assert!(board.is_piece_at_square(Square::F1.as_index(), Piece::Bishop, Color::White));
        assert!(board.is_piece_at_square(Square::G1.as_index(), Piece::Knight, Color::White));
        assert!(board.is_piece_at_square(Square::H1.as_index(), Piece::Rook, Color::White));

        for i in Square::A2 as usize..=Square::H2 as usize {
            assert_eq!(board.squares[i], Some(Piece::Pawn));
            assert_eq!(board.colors[i], Some(Color::White))
        }

        for i in Square::A3 as usize..=Square::H6 as usize {
            assert_eq!(board.squares[i], None);
        }

        for i in Square::A7 as usize..=Square::H7 as usize {
            assert_eq!(board.squares[i], Some(Piece::Pawn));
            assert_eq!(board.colors[i], Some(Color::Black))
        }

        assert!(board.is_piece_at_square(Square::A8.as_index(), Piece::Rook, Color::Black));
        assert!(board.is_piece_at_square(Square::B8.as_index(), Piece::Knight, Color::Black));
        assert!(board.is_piece_at_square(Square::C8.as_index(), Piece::Bishop, Color::Black));
        assert!(board.is_piece_at_square(Square::D8.as_index(), Piece::Queen, Color::Black));
        assert!(board.is_piece_at_square(Square::E8.as_index(), Piece::King, Color::Black));
        assert!(board.is_piece_at_square(Square::F8.as_index(), Piece::Bishop, Color::Black));
        assert!(board.is_piece_at_square(Square::G8.as_index(), Piece::Knight, Color::Black));
        assert!(board.is_piece_at_square(Square::H8.as_index(), Piece::Rook, Color::Black));

        assert_eq!(board.to_move, Color::White);
        assert_eq!(board.en_passant_square, None);
        assert!(board.can_white_king_side_castle);
        assert!(board.can_white_queen_side_castle);
        assert!(board.can_black_king_side_castle);
        assert!(board.can_black_queen_side_castle);
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
            .make_move(Move::from_square(Square::E2, Square::E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::C7, Square::C5, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::G1, Square::F3, Flag::None))
            // TODO: Remove this manual value set when move increment in implemented
            .half_move_clock(1)
            .full_move_number(2)
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
            .piece(Square::D1, Piece::Bishop, Color::Black)
            .piece(Square::A2, Piece::Pawn, Color::White)
            .piece(Square::B2, Piece::Pawn, Color::White)
            .piece(Square::F2, Piece::King, Color::White)
            .piece(Square::H2, Piece::Pawn, Color::White)
            .piece(Square::D4, Piece::Pawn, Color::White)
            .piece(Square::E4, Piece::Pawn, Color::Black)
            .piece(Square::A6, Piece::Pawn, Color::Black)
            .piece(Square::G6, Piece::Pawn, Color::Black)
            .piece(Square::B7, Piece::Pawn, Color::Black)
            .piece(Square::E7, Piece::Pawn, Color::Black)
            .piece(Square::C7, Piece::Rook, Color::White)
            .piece(Square::H7, Piece::Pawn, Color::Black)
            .piece(Square::F8, Piece::King, Color::Black)
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
            .make_move(Move::from_square(Square::E2, Square::E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::E7, Square::E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::G1, Square::F3, Flag::None))
            .make_move(Move::from_square(Square::B8, Square::C6, Flag::None))
            .make_move(Move::from_square(Square::F1, Square::C4, Flag::None))
            // TODO: Remove this manual value set when move increment in implemented
            .half_move_clock(3)
            .full_move_number(3)
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
            .make_move(Move::from_square(Square::E2, Square::E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::C7, Square::C6, Flag::None))
            .make_move(Move::from_square(Square::D2, Square::D4, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::D7, Square::D5, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::E4, Square::E5, Flag::None))
            .make_move(Move::from_square(Square::C8, Square::F5, Flag::None))
            .make_move(Move::from_square(Square::F1, Square::E2, Flag::None))
            .make_move(Move::from_square(Square::E7, Square::E6, Flag::None))
            .make_move(Move::from_square(Square::G1, Square::F3, Flag::None))
            .make_move(Move::from_square(Square::C6, Square::C5, Flag::None))
            .make_move(Move::from_square(Square::C1, Square::E3, Flag::None))
            // TODO: Remove this manual value set when move increment in implemented
            .half_move_clock(1)
            .full_move_number(6)
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
            .make_move(Move::from_square(Square::E2, Square::E4, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::E7, Square::E5, Flag::PawnDoublePush))
            .make_move(Move::from_square(Square::G1, Square::F3, Flag::None))
            .make_move(Move::from_square(Square::B8, Square::C6, Flag::None))
            .make_move(Move::from_square(Square::F1, Square::B5, Flag::None))
            .make_move(Move::from_square(Square::A7, Square::A6, Flag::None))
            .make_move(Move::from_square(Square::B5, Square::A4, Flag::None))
            .make_move(Move::from_square(Square::G8, Square::F6, Flag::None))
            // TODO: Handle castling
            .make_move(Move::from_square(Square::E1, Square::G1, Flag::None))
            .make_move(Move::from_square(Square::H1, Square::F1, Flag::None))
            // end
            .make_move(Move::from_square(Square::F8, Square::E7, Flag::None))
            .make_move(Move::from_square(Square::F1, Square::E1, Flag::None))
            .make_move(Move::from_square(Square::B7, Square::B5, Flag::None))
            .make_move(Move::from_square(Square::A4, Square::B3, Flag::None))
            // TODO: Handle castling
            .make_move(Move::from_square(Square::E8, Square::G8, Flag::None))
            .make_move(Move::from_square(Square::H8, Square::F8, Flag::None))
            // end
            .make_move(Move::from_square(Square::C2, Square::C3, Flag::None))
            .make_move(Move::from_square(Square::D7, Square::D5, Flag::PawnDoublePush))
            // TODO: Remove this manual value set when move increment in implemented
            .half_move_clock(0)
            .full_move_number(9)
            // TODO: Remove this when castling is properly handled
            .can_king_side_castle(Color::White, false)
            .can_king_side_castle(Color::Black, false)
            .can_queen_side_castle(Color::White, false)
            .can_queen_side_castle(Color::Black, false)
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

        board.move_piece(Move::from_square(Square::E2, Square::E4, Flag::PawnDoublePush));
        assert!(board
            .en_passant_square
            .is_some_and(|square| square == Square::E3.as_index()));

        board.move_piece(Move::from_square(Square::E7, Square::E5, Flag::PawnDoublePush));
        assert!(board
            .en_passant_square
            .is_some_and(|square| square == Square::E6.as_index()));

        board.move_piece(Move::from_square(Square::G1, Square::F3, Flag::None));
        assert!(board.en_passant_square.is_none());
    }
}
