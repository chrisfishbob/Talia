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

    pub fn from_symbol(symbol: char) -> Option<Self> {
        match symbol {
            'P' => Some(Self::new(PieceKind::Pawn, Color::White)),
            'p' => Some(Self::new(PieceKind::Pawn, Color::Black)),
            'N' => Some(Self::new(PieceKind::Knight, Color::White)),
            'n' => Some(Self::new(PieceKind::Knight, Color::Black)),
            'B' => Some(Self::new(PieceKind::Bishop, Color::White)),
            'b' => Some(Self::new(PieceKind::Bishop, Color::Black)),
            'R' => Some(Self::new(PieceKind::Rook, Color::White)),
            'r' => Some(Self::new(PieceKind::Rook, Color::Black)),
            'Q' => Some(Self::new(PieceKind::Queen, Color::White)),
            'q' => Some(Self::new(PieceKind::Queen, Color::Black)),
            'K' => Some(Self::new(PieceKind::King, Color::White)),
            'k' => Some(Self::new(PieceKind::King, Color::Black)),
            _ => None,
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
