use core::fmt;

use crate::piece_square_table::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    pub fn to_symbol(&self, color: Color) -> char {
        // TODO: Would unicode chess pieces look better here?
        match (self, color) {
            (Self::Pawn, Color::White) => 'P',
            (Self::Pawn, Color::Black) => 'p',
            (Self::Knight, Color::White) => 'N',
            (Self::Knight, Color::Black) => 'n',
            (Self::Bishop, Color::White) => 'B',
            (Self::Bishop, Color::Black) => 'b',
            (Self::Rook, Color::White) => 'R',
            (Self::Rook, Color::Black) => 'r',
            (Self::Queen, Color::White) => 'Q',
            (Self::Queen, Color::Black) => 'q',
            (Self::King, Color::White) => 'K',
            (Self::King, Color::Black) => 'k',
        }
    }

    pub fn is_sliding_piece(&self) -> bool {
        matches!(self, Piece::Queen | Piece::Rook | Piece::Bishop)
    }

    // TODO: The match can be optimized away using `Piece as usize`
    pub fn piece_value(&self) -> i32 {
        match self {
            Self::Pawn => 100,
            Self::Knight => 300,
            Self::Bishop => 300,
            Self::Rook => 500,
            Self::Queen => 900,
            // King is not included in material count
            Self::King => 0,
        }
    }

    // TODO: The match can be optimized away using `Piece as usize`
    pub fn position_value(&self, square: usize, color: Color) -> i32 {
        let index = match color {
            Color::White => {
                let rank = square / 8;
                let file = square % 8;
                // The piece square tables is inverted from Talia's representation
                let rank = 7 - rank;
                rank * 8 + file
            }
            Color::Black => square,
        };

        match self {
            Self::Pawn => PAWN_SQUARE_TABLE[index],
            Self::Knight => KNIGHT_SQUARE_TABLE[index],
            Self::Bishop => BISHOP_SQUARE_TABLE[index],
            Self::Rook => ROOK_SQUARE_TABLE[index],
            Self::Queen => QUEEN_SQUARE_TABLE[index],
            Self::King => KING_MIDDLE_GAME_SQUARE_TABLE[index],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl fmt::Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::White => write!(f, "White"),
            Self::Black => write!(f, "Black"),
        }
    }
}

impl Color {
    pub fn opposite_color(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}
