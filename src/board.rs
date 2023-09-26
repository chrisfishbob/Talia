use std::collections::HashSet;

use crate::move_generation::Move;
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
    squares: [Option<Piece>; 64],
    to_move: Color,
    can_white_king_side_castle: bool,
    can_black_king_side_castle: bool,
    can_white_queen_side_castle: bool,
    can_black_queen_side_castle: bool,
    en_passant_square: Option<Square>,
    half_move_clock: u32,
    full_move_number: u32,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            squares: [None; 64],
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
        let board_vec: Vec<Vec<char>> = self
            .squares
            .chunks(8)
            .rev()
            .map(|rank| {
                rank.iter()
                    .map(|c| match c {
                        Some(piece) => format!("{piece}").chars().next().unwrap(),
                        None => ' ',
                    })
                    .collect()
            })
            .collect();

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
        write!(f, "{}", self)?;
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

        let mut board: [Option<Piece>; 64] = [None; 64];
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
                    let piece = Piece::from_symbol(piece_char)
                        .ok_or(BoardError::new("invalid piece symbol in FEN"))?;
                    board[rank * 8 + file as usize] = Some(piece);
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
            squares: board,
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

    fn parse_en_passant_square(
        en_passant_sqaure_field: &str,
    ) -> Result<Option<Square>, BoardError> {
        if en_passant_sqaure_field == "-" {
            return Ok(None);
        }

        Ok(Some(Square::from_algebraic_notation(en_passant_sqaure_field)?))
    }

    // TODO: Should this return an error?
    // TODO: Handle en passant, castling, promotion, ...
    pub fn move_piece(&mut self, mv: Move) {
        let starting_piece = self.squares[mv.starting_square as usize];
        self.squares[mv.target_square as usize] = starting_piece;
        self.squares[mv.starting_square as usize] = None;

        if let Color::White = self.to_move {
            self.to_move = Color::Black;
        } else {
            self.to_move = Color::White;
        }
    }

    pub fn set_square(&mut self, square: Square, piece: Option<Piece>) {
        self.squares[square as usize] = piece;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{Board, Square},
        move_generation::Move,
        piece::{Color, Piece, PieceKind},
    };

    #[test]
    fn test_starting_position_board_config() {
        let board = Board::starting_position();
        assert_eq!(
            board.squares[Square::A1 as usize],
            Some(Piece::new(PieceKind::Rook, Color::White))
        );
        assert_eq!(
            board.squares[Square::B1 as usize],
            Some(Piece::new(PieceKind::Knight, Color::White))
        );
        assert_eq!(
            board.squares[Square::C1 as usize],
            Some(Piece::new(PieceKind::Bishop, Color::White))
        );
        assert_eq!(
            board.squares[Square::D1 as usize],
            Some(Piece::new(PieceKind::Queen, Color::White))
        );
        assert_eq!(
            board.squares[Square::E1 as usize],
            Some(Piece::new(PieceKind::King, Color::White))
        );
        assert_eq!(
            board.squares[Square::F1 as usize],
            Some(Piece::new(PieceKind::Bishop, Color::White))
        );
        assert_eq!(
            board.squares[Square::G1 as usize],
            Some(Piece::new(PieceKind::Knight, Color::White))
        );
        assert_eq!(
            board.squares[Square::H1 as usize],
            Some(Piece::new(PieceKind::Rook, Color::White))
        );

        for i in Square::A2 as usize..=Square::H2 as usize {
            assert_eq!(board.squares[i], Some(Piece::new(PieceKind::Pawn, Color::White)));
        }

        for i in Square::A3 as usize..=Square::H6 as usize {
            assert_eq!(board.squares[i], None);
        }

        for i in Square::A7 as usize..=Square::H7 as usize {
            assert_eq!(board.squares[i], Some(Piece::new(PieceKind::Pawn, Color::Black)));
        }

        assert_eq!(
            board.squares[Square::A8 as usize],
            Some(Piece::new(PieceKind::Rook, Color::Black))
        );
        assert_eq!(
            board.squares[Square::B8 as usize],
            Some(Piece::new(PieceKind::Knight, Color::Black))
        );
        assert_eq!(
            board.squares[Square::C8 as usize],
            Some(Piece::new(PieceKind::Bishop, Color::Black))
        );
        assert_eq!(
            board.squares[Square::D8 as usize],
            Some(Piece::new(PieceKind::Queen, Color::Black))
        );
        assert_eq!(
            board.squares[Square::E8 as usize],
            Some(Piece::new(PieceKind::King, Color::Black))
        );
        assert_eq!(
            board.squares[Square::F8 as usize],
            Some(Piece::new(PieceKind::Bishop, Color::Black))
        );
        assert_eq!(
            board.squares[Square::G8 as usize],
            Some(Piece::new(PieceKind::Knight, Color::Black))
        );
        assert_eq!(
            board.squares[Square::H8 as usize],
            Some(Piece::new(PieceKind::Rook, Color::Black))
        );

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
        let mut starting_board = Board::starting_position();
        starting_board.half_move_clock = 1;
        starting_board.full_move_number = 2;
        starting_board.move_piece(Move::new(Square::E2, Square::E4));
        starting_board.move_piece(Move::new(Square::C7, Square::C5));
        starting_board.move_piece(Move::new(Square::G1, Square::F3));

        // Position after 1. e4, c5 => 2. Nf3
        let created_board =
            Board::from_fen("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2")
                .unwrap();

        assert_eq!(starting_board, created_board)
    }

    #[test]
    fn test_from_puzzle_fen() {
        let mut board = Board {
            half_move_clock: 1,
            full_move_number: 31,
            ..Default::default()
        };

        board.set_square(Square::D1, Some(Piece::new(PieceKind::Bishop, Color::Black)));
        board.set_square(Square::A2, Some(Piece::new(PieceKind::Pawn, Color::White)));
        board.set_square(Square::B2, Some(Piece::new(PieceKind::Pawn, Color::White)));
        board.set_square(Square::F2, Some(Piece::new(PieceKind::King, Color::White)));
        board.set_square(Square::H2, Some(Piece::new(PieceKind::Pawn, Color::White)));
        board.set_square(Square::D4, Some(Piece::new(PieceKind::Pawn, Color::White)));
        board.set_square(Square::E4, Some(Piece::new(PieceKind::Pawn, Color::Black)));
        board.set_square(Square::A6, Some(Piece::new(PieceKind::Pawn, Color::Black)));
        board.set_square(Square::G6, Some(Piece::new(PieceKind::Pawn, Color::Black)));
        board.set_square(Square::B7, Some(Piece::new(PieceKind::Pawn, Color::Black)));
        board.set_square(Square::C7, Some(Piece::new(PieceKind::Rook, Color::White)));
        board.set_square(Square::E7, Some(Piece::new(PieceKind::Pawn, Color::Black)));
        board.set_square(Square::H7, Some(Piece::new(PieceKind::Pawn, Color::Black)));
        board.set_square(Square::F8, Some(Piece::new(PieceKind::King, Color::Black)));

        let created_board =
            Board::from_fen("5k2/1pR1p2p/p5p1/8/3Pp3/8/PP3K1P/3b4 w - - 1 31").unwrap();

        assert_eq!(board, created_board);
    }

    #[test]
    fn test_from_fen_invalid_piece_position_char() {
        let board = Board::from_fen("9/8/8/8/8/8/8/8 w - - 0 1");

        assert_eq!(board.err().unwrap().to_string(), "invalid piece symbol in FEN")
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
        assert_eq!(index.unwrap(), Some(Square::A1));
    }

    #[test]
    fn test_parse_en_passant_square_e4() {
        let field = "e4";
        let index = Board::parse_en_passant_square(field);
        assert_eq!(index.unwrap(), Some(Square::E4));
    }

    #[test]
    fn test_parse_en_passant_square_f7() {
        let field = "f7";
        let index = Board::parse_en_passant_square(field);
        assert_eq!(index.unwrap(), Some(Square::F7));
    }

    #[test]
    fn test_parse_en_passant_square_h8() {
        let field = "h8";
        let index = Board::parse_en_passant_square(field);
        assert_eq!(index.unwrap(), Some(Square::H8));
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
