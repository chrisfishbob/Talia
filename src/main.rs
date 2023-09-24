pub mod board;
use crate::board::Board;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Piece {
    None,
    Pawn(Color),
    Knight(Color),
    Bishop(Color),
    Rook(Color),
    Queen(Color),
    King(Color),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    White,
    Black,
}

fn main() {
    let board = Board::default_config();
    println!("Lubyanka Chess Engine: v0.0.1");
    println!("{:#?}", board)
}
