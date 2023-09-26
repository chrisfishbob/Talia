use core::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    None,
    Pawn(Color),
    Knight(Color),
    Bishop(Color),
    Rook(Color),
    Queen(Color),
    King(Color),
}

impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => write!(f, " "),
            Self::Pawn(Color::White) => write!(f, "P"),
            Self::Pawn(Color::Black) => write!(f, "p"),
            Self::Knight(Color::White) => write!(f, "N"),
            Self::Knight(Color::Black) => write!(f, "n"),
            Self::Bishop(Color::White) => write!(f, "B"),
            Self::Bishop(Color::Black) => write!(f, "b"),
            Self::Rook(Color::White) => write!(f, "R"),
            Self::Rook(Color::Black) => write!(f, "r"),
            Self::Queen(Color::White) => write!(f, "Q"),
            Self::Queen(Color::Black) => write!(f, "q"),
            Self::King(Color::White) => write!(f, "K"),
            Self::King(Color::Black) => write!(f, "k"),
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
