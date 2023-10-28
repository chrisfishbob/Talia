use crate::{move_generation::MoveGenerator, piece::Color};

pub fn evaluate(move_generator: &MoveGenerator) -> i32 {
    let white_eval = count_material(move_generator, Color::White);
    let black_eval = count_material(move_generator, Color::Black);

    let net_eval = white_eval - black_eval;

    if move_generator.board.to_move == Color::White {net_eval} else {-net_eval}
}

fn count_material(move_generator: &MoveGenerator, color: Color) -> i32 {
    let mut count = 0;
    for square in 0..64 {
        match move_generator.board.colors[square] {
            Some(c) if c == color => {
                count += move_generator.board.squares[square].unwrap().piece_value();
            },
            _ => continue,
        }
    }

    count
}
