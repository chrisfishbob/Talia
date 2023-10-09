use crate::board::Board;
use crate::move_generation::Move;
use crate::errors::BoardError;
use crate::piece::{Color, Piece};
use crate::square::Square;
use std::collections::HashSet;

pub struct BoardBuilder {
    board: Board,
}

impl BoardBuilder {
    pub fn new() -> Self {
        Self {
            board: Board::default(),
        }
    }

    pub fn from_starting_position() -> Self {
        Self { board: Board::starting_position() }
    }

    pub fn build_from(board: Board) -> Self {
        Self { board }
    }

    pub fn make_move(mut self, mv: Move) -> Self {
        self.board.move_piece(mv);
        self
    }

    pub fn piece(mut self, square: Square, piece: Piece, color: Color) -> Self {
        self.board.put_piece(square.as_index(), piece, color);
        self
    }

    pub fn to_move(mut self, color: Color) -> Self {
        self.board.to_move = color;
        self
    }

    pub fn can_king_side_castle(mut self, color: Color, castle: bool) -> Self {
        if color == Color::White {
            self.board.can_black_king_side_castle = castle;
        } else {
            self.board.can_white_king_side_castle = castle;
        }
        self
    }

    pub fn can_queen_side_castle(mut self, color: Color, castle: bool) -> Self {
        if color == Color::White {
            self.board.can_black_queen_side_castle = castle;
        } else {
            self.board.can_white_queen_side_castle = castle;
        }
        self
    }

    pub fn en_passant_square(mut self, square: Option<usize>) -> Self {
        self.board.en_passant_square = square;
        self
    }

    pub fn half_move_clock(mut self, number: u32) -> Self {
        self.board.half_move_clock = number;
        self
    }

    pub fn full_move_number(mut self, number: u32) -> Self {
        self.board.full_move_number = number;
        self
    }

    pub fn try_from_fen(fen: &str) -> Result<Board, BoardError> {
        // 0: board arrangement
        // 1: active color
        // 2: Castling availability
        // 3: En passant square
        // 4: Halfmove clock
        // 5: Fullmove number
        let fen_string_fields: Vec<&str> = fen.split_whitespace().collect();

        let mut squares: [Option<Piece>; 64] = [None; 64];
        let mut colors: [Option<Color>; 64] = [None; 64];
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
                    let (piece, color) = match piece_char {
                        'P' => (Piece::Pawn, Color::White),
                        'p' => (Piece::Pawn, Color::Black),
                        'N' => (Piece::Knight, Color::White),
                        'n' => (Piece::Knight, Color::Black),
                        'B' => (Piece::Bishop, Color::White),
                        'b' => (Piece::Bishop, Color::Black),
                        'R' => (Piece::Rook, Color::White),
                        'r' => (Piece::Rook, Color::Black),
                        'Q' => (Piece::Queen, Color::White),
                        'q' => (Piece::Queen, Color::Black),
                        'K' => (Piece::King, Color::White),
                        'k' => (Piece::King, Color::Black),
                        _ => Err(BoardError::new("invalid piece symbol in FEN"))?,
                    };

                    let index = rank * 8 + file as usize;
                    squares[index] = Some(piece);
                    colors[index] = Some(color);

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

        Ok(Board {
            squares,
            colors,
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
            Square::from_algebraic_notation(en_passant_sqaure_field)?.as_index(),
        ))
    }
}

impl TryInto<Board> for BoardBuilder {
    type Error = BoardError;
    fn try_into(self) -> Result<Board, Self::Error> {
        // TODO: Add checks for invalid board states
        Ok(self.board)
    }
}

impl Default for BoardBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::board_builder::BoardBuilder;
    use crate::square::Square;

    #[test]
    fn test_from_fen_invalid_piece_position_char() {
        let board = BoardBuilder::try_from_fen("9/8/8/8/8/8/8/8 w - - 0 1");

        assert_eq!(board.err().unwrap().to_string(), "invalid piece symbol in FEN")
    }

    #[test]
    fn test_from_fen_invalid_to_move_color() {
        let board = BoardBuilder::try_from_fen("8/8/8/8/8/8/8/8 - - - 0 1");

        assert_eq!(
            board.err().unwrap().to_string(),
            "failed to parse active board color, must be 'b' or 'w'."
        )
    }

    #[test]
    fn test_from_fen_invalid_half_move_clock() {
        let board = BoardBuilder::try_from_fen("8/8/8/8/8/8/8/8 w - - -1 1");

        assert_eq!(
            board.err().unwrap().to_string(),
            "failed to parse half move clock from fen"
        )
    }

    #[test]
    fn test_from_fen_invalid_full_move_number() {
        let board = BoardBuilder::try_from_fen("8/8/8/8/8/8/8/8 w - - 1 -1");

        assert_eq!(
            board.err().unwrap().to_string(),
            "failed to parse full move number from fen"
        )
    }

    #[test]
    fn test_from_fen_invalid_castling_rights() {
        let board = BoardBuilder::try_from_fen("8/8/8/8/8/8/8/8 w bw - 1 1");

        assert_eq!(
            board.err().unwrap().to_string(),
            "invalid castling rights in fen, must be a combination of 'K', 'Q', 'k', and 'q' or '-'"
        )
    }

    #[test]
    fn test_parse_en_passant_square_none() {
        let field = "-";
        let index = BoardBuilder::parse_en_passant_square(field);
        assert_eq!(index.unwrap(), None);
    }

    #[test]
    fn test_parse_en_passant_square_a1() {
        let field = "a1";
        let index = BoardBuilder::parse_en_passant_square(field);
        assert_eq!(index.unwrap(), Some(Square::A1 as usize));
    }

    #[test]
    fn test_parse_en_passant_square_e4() {
        let field = "e4";
        let index = BoardBuilder::parse_en_passant_square(field);
        assert_eq!(index.unwrap(), Some(Square::E4 as usize));
    }

    #[test]
    fn test_parse_en_passant_square_f7() {
        let field = "f7";
        let index = BoardBuilder::parse_en_passant_square(field);
        assert_eq!(index.unwrap(), Some(Square::F7 as usize));
    }

    #[test]
    fn test_parse_en_passant_square_h8() {
        let field = "h8";
        let index = BoardBuilder::parse_en_passant_square(field);
        assert_eq!(index.unwrap(), Some(Square::H8 as usize));
    }

    #[test]
    fn test_parse_en_passant_square_invalid_file() {
        let field = "-7";
        let index = BoardBuilder::parse_en_passant_square(field);
        assert!(index.is_err());
        assert_eq!(index.err().unwrap().to_string(), "Invalid square string: -7")
    }

    #[test]
    fn test_parse_en_passant_square_missing_rank() {
        let field = "h";
        let index = BoardBuilder::parse_en_passant_square(field);
        assert!(index.is_err());
        assert_eq!(index.err().unwrap().to_string(), "Invalid square string: h")
    }

    #[test]
    fn test_parse_en_passant_square_invalid_rank() {
        let field = "hh";
        let index = BoardBuilder::parse_en_passant_square(field);
        assert!(index.is_err());
        assert_eq!(index.err().unwrap().to_string(), "Invalid square string: hh")
    }
}
