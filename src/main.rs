pub mod board;
pub mod piece;
pub mod square;
pub mod move_generation;
use crate::board::Board;

fn main() {
    let board = Board::starting_position();
    println!("Talia Chess Engine: v0.0.1");
    // println!("{board}")
    println!("{:?}", board);
}
