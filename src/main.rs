pub mod board;
pub mod piece;
pub mod square;
use crate::board::Board;

fn main() {
    let board = Board::default_config();
    println!("Lubyanka Chess Engine: v0.0.1");
    println!("{:#?}", board)
}
