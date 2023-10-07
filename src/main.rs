pub mod board;
pub mod move_generation;
pub mod piece;
pub mod square;
use crate::board::Board;
use crate::move_generation::{Move, MoveGenerator, Flag};
use crate::square::Square;

fn main() {
    let mut board = Board::starting_position();
    board.move_piece(Move::from_square(Square::E2, Square::E4, Flag::None));
    board.move_piece(Move::from_square(Square::E7, Square::E5, Flag::None));
    board.move_piece(Move::from_square(Square::G1, Square::F3, Flag::None));
    board.move_piece(Move::from_square(Square::B8, Square::C6, Flag::None));
    println!("Talia Chess Engine: v0.0.1");
    println!("{board}");

    let mut generator = MoveGenerator::new(board);
    generator.generate_moves();

    for mv in generator.moves {
        println!("{:?}", mv);
    }
}
