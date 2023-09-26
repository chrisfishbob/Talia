use core::fmt;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Piece {
    piece_kind: PieceKind,
    color: Color,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.piece_kind, self.color) {
            (PieceKind::Pawn, Color::White) => write!(f, "P"),
            (PieceKind::Pawn, Color::Black) => write!(f, "p"),
            (PieceKind::Knight, Color::White) => write!(f, "N"),
            (PieceKind::Knight, Color::Black) => write!(f, "n"),
            (PieceKind::Bishop, Color::White) => write!(f, "B"),
            (PieceKind::Bishop, Color::Black) => write!(f, "b"),
            (PieceKind::Rook, Color::White) => write!(f, "R"),
            (PieceKind::Rook, Color::Black) => write!(f, "r"),
            (PieceKind::Queen, Color::White) => write!(f, "Q"),
            (PieceKind::Queen, Color::Black) => write!(f, "q"),
            (PieceKind::King, Color::White) => write!(f, "K"),
            (PieceKind::King, Color::Black) => write!(f, "k"),
        }
    }
}

impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Piece {
    pub fn new(piece_kind: PieceKind, color: Color) -> Self {
        Self { piece_kind, color }
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
