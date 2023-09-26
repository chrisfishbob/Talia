use std::collections::{HashMap, HashSet};

use crate::piece::{Color, Piece};
use crate::square::Square;
use std::{error, fmt};

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

#[derive(PartialEq, Eq)]
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

impl Default for Board {
    fn default() -> Self {
        Self {
            board: [Piece::None; 64],
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

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut board_vec: Vec<Vec<Piece>> = Vec::new();
        for rank in self.board.chunks(8) {
            board_vec.insert(0, rank.to_vec());
        }

        writeln!(f)?;
        for (i, rank) in board_vec.iter().enumerate() {
            let rank_num = 8 - i; 
            writeln!(f, "{rank_num}  {:?}", rank)?;
        }

        writeln!(f, "\n    A  B  C  D  E  F  G  H\n")?;
        writeln!(f, "{:?} to move.", self.to_move)
    }
}

impl Board {
    pub fn starting_position() -> Self {
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
                        .ok_or(BoardError::new("Invalid piece position char in FEN string"))?;

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

        let valid_casting_right_chars: HashSet<char> =
            ['K', 'Q', 'k', 'q', '-'].iter().cloned().collect();
        let castling_rights: HashSet<char> = fen_string_fields[2].chars().collect();
        if !castling_rights.is_subset(&valid_casting_right_chars) {
            return Err(BoardError::new(
                "invalid castling rights in fen, must be a combination of 'K', 'Q', 'k', and 'q' or '-'",
            ));
        }

        let half_move_clock: u32 = fen_string_fields[4]
            .parse()
            .map_err(|_| BoardError::new("failed to parse half move clock from fen"))?;

        let full_move_number: u32 = fen_string_fields[5]
            .parse()
            .map_err(|_| BoardError::new("failed to parse full move number from fen"))?;

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
            Square::from_algebraic_notation(en_passant_sqaure_field)? as usize
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
    fn test_starting_position_board_config() {
        let board = Board::starting_position();
        assert_eq!(board.board[Square::A1 as usize], Piece::Rook(Color::White));
        assert_eq!(board.board[Square::B1 as usize], Piece::Knight(Color::White));
        assert_eq!(board.board[Square::C1 as usize], Piece::Bishop(Color::White));
        assert_eq!(board.board[Square::D1 as usize], Piece::Queen(Color::White));
        assert_eq!(board.board[Square::E1 as usize], Piece::King(Color::White));
        assert_eq!(board.board[Square::F1 as usize], Piece::Bishop(Color::White));
        assert_eq!(board.board[Square::G1 as usize], Piece::Knight(Color::White));
        assert_eq!(board.board[Square::H1 as usize], Piece::Rook(Color::White));

        for i in Square::A2 as usize..=Square::H2 as usize {
            assert_eq!(board.board[i], Piece::Pawn(Color::White));
        }

        for i in Square::A3 as usize..=Square::H6 as usize {
            assert_eq!(board.board[i], Piece::None);
        }

        for i in Square::A7 as usize..=Square::H7 as usize {
            assert_eq!(board.board[i], Piece::Pawn(Color::Black));
        }

        assert_eq!(board.board[Square::A8 as usize], Piece::Rook(Color::Black));
        assert_eq!(board.board[Square::B8 as usize], Piece::Knight(Color::Black));
        assert_eq!(board.board[Square::C8 as usize], Piece::Bishop(Color::Black));
        assert_eq!(board.board[Square::D8 as usize], Piece::Queen(Color::Black));
        assert_eq!(board.board[Square::E8 as usize], Piece::King(Color::Black));
        assert_eq!(board.board[Square::F8 as usize], Piece::Bishop(Color::Black));
        assert_eq!(board.board[Square::G8 as usize], Piece::Knight(Color::Black));
        assert_eq!(board.board[Square::H8 as usize], Piece::Rook(Color::Black));

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
    fn test_from_fen_empty_board() {
        let empty_board = Board::default();
        let empty_board_from_fen = Board::from_fen("8/8/8/8/8/8/8/8 w - - 0 1").unwrap();

        assert_eq!(empty_board, empty_board_from_fen);
    }

    #[test]
    fn test_from_fen_sicilian_defense() {
        let mut statring_board = Board::starting_position();
        statring_board.to_move = Color::Black;
        statring_board.half_move_clock = 1;
        statring_board.full_move_number = 2;
        statring_board.place_piece(Square::E2, Piece::None);
        statring_board.place_piece(Square::E2, Piece::None);
        statring_board.place_piece(Square::E4, Piece::Pawn(Color::White));
        statring_board.place_piece(Square::C7, Piece::None);
        statring_board.place_piece(Square::C5, Piece::Pawn(Color::Black));
        statring_board.place_piece(Square::G1, Piece::None);
        statring_board.place_piece(Square::F3, Piece::Knight(Color::White));

        // Position after 1. e4, c5 => 2. Nf3
        let created_board =
            Board::from_fen("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2")
                .unwrap();

        assert_eq!(statring_board, created_board)
    }

    #[test]
    fn test_from_puzzle_fen() {
        let mut board = Board {
            half_move_clock: 1,
            full_move_number: 31,
            ..Default::default()
        };

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
    fn test_from_fen_invalid_piece_position_char() {
        let board = Board::from_fen("9/8/8/8/8/8/8/8 w - - 0 1");

        assert_eq!(
            board.err().unwrap().to_string(),
            "Invalid piece position char in FEN string"
        )
    }

    #[test]
    fn test_from_fen_invalid_to_move_color() {
        let board = Board::from_fen("8/8/8/8/8/8/8/8 - - - 0 1");

        assert_eq!(
            board.err().unwrap().to_string(),
            "failed to parse active board color, must be 'b' or 'w'."
        )
    }

    #[test]
    fn test_from_fen_invalid_half_move_clock() {
        let board = Board::from_fen("8/8/8/8/8/8/8/8 w - - -1 1");

        assert_eq!(
            board.err().unwrap().to_string(),
            "failed to parse half move clock from fen"
        )
    }

    #[test]
    fn test_from_fen_invalid_full_move_number() {
        let board = Board::from_fen("8/8/8/8/8/8/8/8 w - - 1 -1");

        assert_eq!(
            board.err().unwrap().to_string(),
            "failed to parse full move number from fen"
        )
    }

    #[test]
    fn test_from_fen_invalid_castling_rights() {
        let board = Board::from_fen("8/8/8/8/8/8/8/8 w bw - 1 1");

        assert_eq!(
            board.err().unwrap().to_string(),
            "invalid castling rights in fen, must be a combination of 'K', 'Q', 'k', and 'q' or '-'"
        )
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
        assert_eq!(index.err().unwrap().to_string(), "Invalid square string: -7")
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
        assert_eq!(index.err().unwrap().to_string(), "Invalid square string: hh")
    }
}
