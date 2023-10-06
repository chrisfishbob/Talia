use core::fmt;

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
