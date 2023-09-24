use std::collections::HashMap;

use crate::{Color, Piece};
use std::{error, fmt};

#[derive(Debug)]
pub struct Board {
    board: [Piece; 64],
    to_move: Color,
    en_passant_square: Option<usize>,
    can_white_king_side_castle: bool,
    can_black_king_side_castle: bool,
    can_white_queen_side_castle: bool,
    can_black_queen_side_castle: bool,
    half_move_clock: u32,
    full_move_number: u32,
}

#[derive(Debug, Clone)]
pub struct FenParseError {
    message: String,
}

impl FenParseError {
    pub fn new(message: &str) -> FenParseError {
        FenParseError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for FenParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for FenParseError {}

impl Board {
    pub fn default_config() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("failed to construct default board config")
    }

    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
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
            if symbol == '/' {
                file = 0;
                rank -= 1;
                continue;
            }

            if let Some(digit) = symbol.to_digit(10) {
                file += digit;
            } else {
                board[rank * 8 + file as usize] = symbol_to_piece[&symbol];
                file += 1;
            }
        }

        let to_move = match fen_string_fields[1] {
            "w" => Color::White,
            "b" => Color::Black,
            other => {
                return Err(FenParseError::new(
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
            .map_err(|_| FenParseError::new("failed to parse half move clock from fen"))?;

        let full_move_number: u32 = fen_string_fields[5]
            .parse()
            .map_err(|_| FenParseError::new("failed to parse full move clock from fen"))?;

        Ok(Self {
            board,
            to_move,
            en_passant_square: parse_en_passant_square(fen_string_fields[3])?,
            can_white_king_side_castle: castling_rights.contains(&'K'),
            can_black_king_side_castle: castling_rights.contains(&'k'),
            can_white_queen_side_castle: castling_rights.contains(&'Q'),
            can_black_queen_side_castle: castling_rights.contains(&'q'),
            half_move_clock,
            full_move_number,
        })
    }
}

fn parse_en_passant_square(en_passant_sqaure_field: &str) -> Result<Option<usize>, FenParseError> {
    if en_passant_sqaure_field == "-" {
        return Ok(None);
    }

    let mut en_passant_square_chars = en_passant_sqaure_field.chars();

    let file_char = en_passant_square_chars.next().unwrap();
    if !file_char.is_ascii_lowercase() {
        return Err(FenParseError::new("invalid file provided, should be a-z"));
    }
    let file_num = file_char as usize - 'a' as usize;

    let rank = en_passant_square_chars
        .next()
        .ok_or(FenParseError::new("failed to get rank from en passant field"))?
        .to_digit(10)
        .ok_or(FenParseError::new("failed to parse rank to a valid integer"))? as usize
        - 1;

    Ok(Some(rank * 8 + file_num))
}

#[cfg(test)]
mod tests {
    use crate::board::parse_en_passant_square;

    #[test]
    fn test_parse_en_passant_square_none() {
        let field = "-";
        let index = parse_en_passant_square(field);
        assert_eq!(index.unwrap(), None);
    }

    #[test]
    fn test_parse_en_passant_square_a1() {
        let field = "a1";
        let index = parse_en_passant_square(field);
        assert_eq!(index.unwrap(), Some(0));
    }

    #[test]
    fn test_parse_en_passant_square_e4() {
        let field = "e4";
        let index = parse_en_passant_square(field);
        assert_eq!(index.unwrap(), Some(28));
    }

    #[test]
    fn test_parse_en_passant_square_f7() {
        let field = "f7";
        let index = parse_en_passant_square(field);
        assert_eq!(index.unwrap(), Some(53));
    }
}
