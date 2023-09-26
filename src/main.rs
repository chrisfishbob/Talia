pub mod board;
pub mod piece;
pub mod square;
pub mod move_generation;
use crate::board::Board;
use crate::square::Square;
use crate::move_generation::Move;

fn main() {
    let mut board = Board::starting_position();
    board.move_piece(Move::new(Square::E2, Square::E4));
    board.move_piece(Move::new(Square::E7, Square::E5));
    board.move_piece(Move::new(Square::G1, Square::F3));
    board.move_piece(Move::new(Square::B8, Square::C6));
    println!("Talia Chess Engine: v0.0.1");
    println!("{board}")
}
