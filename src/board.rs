use std::collections::HashMap;

use crate::piece::{Color, Piece};
use crate::square::Square;
use std::{error, fmt};

#[derive(Debug, PartialEq, Eq)]
pub struct Board {
    board: [Piece; 64],
    to_move: Color,
    can_white_king_side_castle: bool,
    can_black_king_side_castle: bool,
    can_white_queen_side_castle: bool,
    can_black_queen_side_castle: bool,
    en_passant_square: Option<usize>,
    half_move_clock: u32,
    full_move_number: u32,
}

#[derive(Debug, Clone)]
pub struct BoardError {
    message: String,
}

impl BoardError {
    pub fn new(message: &str) -> BoardError {
        BoardError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for BoardError {}

impl Board {
    pub fn blank_board() -> Self {
        Self {
            board: [Piece::None; 64],
            to_move: Color::White,
            can_white_king_side_castle: true,
            can_white_queen_side_castle: true,
            can_black_king_side_castle: true,
            can_black_queen_side_castle: true,
            en_passant_square: None,
            half_move_clock: 0,
            full_move_number: 1,
        }
    }

    pub fn default_config() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("failed to construct default board config")
    }

    pub fn from_fen(fen: &str) -> Result<Self, BoardError> {
        // 0: board arrangement
        // 1: active color
        // 2: Castling availability
        // 3: En passant square
        // 4: Halfmove clock
        // 5: Fullmove number
        let fen_string_fields: Vec<&str> = fen.split_whitespace().collect();

        let mut symbol_to_piece = HashMap::new();
        symbol_to_piece.insert('k', Piece::King(Color::Black));
        symbol_to_piece.insert('q', Piece::Queen(Color::Black));
        symbol_to_piece.insert('r', Piece::Rook(Color::Black));
        symbol_to_piece.insert('n', Piece::Knight(Color::Black));
        symbol_to_piece.insert('b', Piece::Bishop(Color::Black));
        symbol_to_piece.insert('p', Piece::Pawn(Color::Black));
        symbol_to_piece.insert('K', Piece::King(Color::White));
        symbol_to_piece.insert('Q', Piece::Queen(Color::White));
        symbol_to_piece.insert('R', Piece::Rook(Color::White));
        symbol_to_piece.insert('N', Piece::Knight(Color::White));
        symbol_to_piece.insert('B', Piece::Bishop(Color::White));
        symbol_to_piece.insert('P', Piece::Pawn(Color::White));

        let mut board: [Piece; 64] = [Piece::None; 64];
        let mut file = 0;
        let mut rank = 7;

        for symbol in fen_string_fields[0].chars() {
            match symbol {
                '/' => {
                    file = 0;
                    rank -= 1;
                }
                '1'..='8' => file += symbol.to_digit(10).unwrap(),
                piece_char => {
                    let piece = symbol_to_piece
                        .get(&piece_char)
                        .ok_or(BoardError::new("Invalid piece char in FEN string"))?;

                    board[rank * 8 + file as usize] = *piece;
                    file += 1;
                }
            }
        }

        let to_move = match fen_string_fields[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => {
                return Err(BoardError::new(
                    "failed to parse active board color, must be 'b' or 'w'.",
                ))
            }
        };

        let mut castling_rights = Vec::new();
        for char in fen_string_fields[2].chars() {
            castling_rights.push(char);
        }

        let half_move_clock: u32 = fen_string_fields[4]
            .parse()
            .map_err(|_| BoardError::new("failed to parse half move clock from fen"))?;

        let full_move_number: u32 = fen_string_fields[5]
            .parse()
            .map_err(|_| BoardError::new("failed to parse full move clock from fen"))?;

        Ok(Self {
            board,
            to_move,
            en_passant_square: Self::parse_en_passant_square(fen_string_fields[3])?,
            can_white_king_side_castle: castling_rights.contains(&'K'),
            can_black_king_side_castle: castling_rights.contains(&'k'),
            can_white_queen_side_castle: castling_rights.contains(&'Q'),
            can_black_queen_side_castle: castling_rights.contains(&'q'),
            half_move_clock,
            full_move_number,
        })
    }

    fn parse_en_passant_square(en_passant_sqaure_field: &str) -> Result<Option<usize>, BoardError> {
        if en_passant_sqaure_field == "-" {
            return Ok(None);
        }

        Ok(Some(
            Square::from_algebraic_notation(en_passant_sqaure_field)? as usize,
        ))
    }

    pub fn place_piece(&mut self, square: Square, piece: Piece) {
        self.board[square as usize] = piece;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{Board, Square},
        piece::{Color, Piece},
    };

    #[test]
    fn test_default_board_config() {
        let board = Board::default_config();
        assert_eq!(board.board[0], Piece::Rook(Color::White));
        assert_eq!(board.board[1], Piece::Knight(Color::White));
        assert_eq!(board.board[2], Piece::Bishop(Color::White));
        assert_eq!(board.board[3], Piece::Queen(Color::White));
        assert_eq!(board.board[4], Piece::King(Color::White));
        assert_eq!(board.board[5], Piece::Bishop(Color::White));
        assert_eq!(board.board[6], Piece::Knight(Color::White));
        assert_eq!(board.board[7], Piece::Rook(Color::White));

        for i in 8..=15 {
            assert_eq!(board.board[i], Piece::Pawn(Color::White));
        }

        for i in 16..=47 {
            assert_eq!(board.board[i], Piece::None);
        }

        for i in 48..=55 {
            assert_eq!(board.board[i], Piece::Pawn(Color::Black));
        }

        assert_eq!(board.board[56], Piece::Rook(Color::Black));
        assert_eq!(board.board[57], Piece::Knight(Color::Black));
        assert_eq!(board.board[58], Piece::Bishop(Color::Black));
        assert_eq!(board.board[59], Piece::Queen(Color::Black));
        assert_eq!(board.board[60], Piece::King(Color::Black));
        assert_eq!(board.board[61], Piece::Bishop(Color::Black));
        assert_eq!(board.board[62], Piece::Knight(Color::Black));
        assert_eq!(board.board[63], Piece::Rook(Color::Black));

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
    fn test_from_fen_sicilian_defense() {
        let mut default_board = Board::default_config();
        default_board.to_move = Color::Black;
        default_board.half_move_clock = 1;
        default_board.full_move_number = 2;
        default_board.place_piece(Square::E2, Piece::None);
        default_board.place_piece(Square::E2, Piece::None);
        default_board.place_piece(Square::E4, Piece::Pawn(Color::White));
        default_board.place_piece(Square::C7, Piece::None);
        default_board.place_piece(Square::C5, Piece::Pawn(Color::Black));
        default_board.place_piece(Square::G1, Piece::None);
        default_board.place_piece(Square::F3, Piece::Knight(Color::White));

        // Position after 1. e4, c5 => 2. Nf3
        let created_board =
            Board::from_fen("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2")
                .unwrap();

        assert_eq!(default_board, created_board)
    }

    #[test]
    fn test_from_puzzle_fen() {
        let mut board = Board::blank_board();
        board.can_white_king_side_castle = false;
        board.can_white_queen_side_castle = false;
        board.can_black_king_side_castle = false;
        board.can_black_queen_side_castle = false;
        board.half_move_clock = 1;
        board.full_move_number = 31;
        board.place_piece(Square::D1, Piece::Bishop(Color::Black));
        board.place_piece(Square::A2, Piece::Pawn(Color::White));
        board.place_piece(Square::B2, Piece::Pawn(Color::White));
        board.place_piece(Square::F2, Piece::King(Color::White));
        board.place_piece(Square::H2, Piece::Pawn(Color::White));
        board.place_piece(Square::D4, Piece::Pawn(Color::White));
        board.place_piece(Square::E4, Piece::Pawn(Color::Black));
        board.place_piece(Square::A6, Piece::Pawn(Color::Black));
        board.place_piece(Square::G6, Piece::Pawn(Color::Black));
        board.place_piece(Square::B7, Piece::Pawn(Color::Black));
        board.place_piece(Square::C7, Piece::Rook(Color::White));
        board.place_piece(Square::E7, Piece::Pawn(Color::Black));
        board.place_piece(Square::H7, Piece::Pawn(Color::Black));
        board.place_piece(Square::F8, Piece::King(Color::Black));

        let created_board =
            Board::from_fen("5k2/1pR1p2p/p5p1/8/3Pp3/8/PP3K1P/3b4 w - - 1 31").unwrap();

        assert_eq!(board, created_board);
    }

    #[test]
    fn test_parse_en_passant_square_none() {
        let field = "-";
        let index = Board::parse_en_passant_square(field);
        assert_eq!(index.unwrap(), None);
    }

    #[test]
    fn test_parse_en_passant_square_a1() {
        let field = "a1";
        let index = Board::parse_en_passant_square(field);
        assert_eq!(index.unwrap(), Some(0));
    }

    #[test]
    fn test_parse_en_passant_square_e4() {
        let field = "e4";
        let index = Board::parse_en_passant_square(field);
        assert_eq!(index.unwrap(), Some(28));
    }

    #[test]
    fn test_parse_en_passant_square_f7() {
        let field = "f7";
        let index = Board::parse_en_passant_square(field);
        assert_eq!(index.unwrap(), Some(53));
    }

    #[test]
    fn test_parse_en_passant_square_h8() {
        let field = "h8";
        let index = Board::parse_en_passant_square(field);
        assert_eq!(index.unwrap(), Some(63));
    }

    #[test]
    fn test_parse_en_passant_square_invalid_file() {
        let field = "-7";
        let index = Board::parse_en_passant_square(field);
        assert!(index.is_err());
        assert_eq!(
            index.err().unwrap().to_string(),
            "Invalid square string: -7"
        )
    }

    #[test]
    fn test_parse_en_passant_square_missing_rank() {
        let field = "h";
        let index = Board::parse_en_passant_square(field);
        assert!(index.is_err());
        assert_eq!(index.err().unwrap().to_string(), "Invalid square string: h")
    }

    #[test]
    fn test_parse_en_passant_square_invalid_rank() {
        let field = "hh";
        let index = Board::parse_en_passant_square(field);
        assert!(index.is_err());
        assert_eq!(
            index.err().unwrap().to_string(),
            "Invalid square string: hh"
        )
    }
}
