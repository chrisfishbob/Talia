// I know, I know, will remove once this is more than 0.0001% done.
// The LSP is being annoying right now.
#![allow(dead_code, unused_variables)]
pub mod board;
use crate::board::Board;

#[derive(Debug, Clone, Copy)]
enum Piece {
    None,
    Pawn(Color),
    Knight(Color),
    Bishop(Color),
    Rook(Color),
    Queen(Color),
    King(Color),
}

#[derive(Debug, Clone, Copy)]
enum Color {
    White,
    Black,
}

fn main() {
    let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq @ 0 1");
    match board {
        Ok(b) => println!("We good!"),
        Err(e) => println!("Oh no: {e}"),
    }
    // println!("Hello, world!");
    // println!("The board is {:#?}", board);
}
