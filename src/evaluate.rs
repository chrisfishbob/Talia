use crate::{
    move_generation::MoveGenerator,
    piece::{Color, Piece},
};

// Source: https://www.chessprogramming.org/Simplified_Evaluation_Function

#[allow(unused)]
#[rustfmt::skip]
const WHITE_PAWN_SQUARE_TABLE: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     50, 50, 50, 50, 50, 50, 50, 50,
     10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0
];

#[allow(unused)]
#[rustfmt::skip]
const BLACK_PAWN_SQUARE_TABLE: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10,-20,-20, 10, 10,  5,
     5, -5,-10,  0,  0,-10, -5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5,  5, 10, 25, 25, 10,  5,  5,
     10, 10, 20, 30, 30, 20, 10, 10,
     50, 50, 50, 50, 50, 50, 50, 50,
     0,  0,  0,  0,  0,  0,  0,  0
];

#[allow(unused)]
#[rustfmt::skip]
const KNIGHT_SQUARE_TABLE: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[allow(unused)]
#[rustfmt::skip]
const WHITE_BISHOP_SQUARE_TABLE: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[allow(unused)]
#[rustfmt::skip]
const BLACK_BISHOP_SQUARE_TABLE: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[allow(unused)]
#[rustfmt::skip]
const WHITE_ROOK_SQUARE_TABLE: [i32; 64] = [
      0,  0,  0,  0,  0,  0,  0,  0,
      5, 10, 10, 10, 10, 10, 10,  5,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
      0,  0,  0,  5,  5,  0,  0,  0
];

#[allow(unused)]
#[rustfmt::skip]
const BLACK_ROOK_SQUARE_TABLE: [i32; 64] = [
      0,  0,  0,  5,  5,  0,  0,  0,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
      5, 10, 10, 10, 10, 10, 10,  5,
      0,  0,  0,  0,  0,  0,  0,  0,
];

#[allow(unused)]
#[rustfmt::skip]
const WHITE_QUEEN_SQUARE_TABLE: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];

#[allow(unused)]
#[rustfmt::skip]
const BLACK_QUEEN_SQUARE_TABLE: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -10,  5,  5,  5,  5,  5,  0,-10,
      0,  0,  5,  5,  5,  5,  0, -5,
     -5,  0,  5,  5,  5,  5,  0, -5,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];

#[allow(unused)]
#[rustfmt::skip]
const WHITE_KING_MIDDLE_GAME_SQUARE_TABLE: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20
];

#[allow(unused)]
#[rustfmt::skip]
const BLACK_KING_MIDDLE_GAME_SQUARE_TABLE: [i32; 64] = [
     20, 30, 10,  0,  0, 10, 30, 20,
     20, 20,  0,  0,  0,  0, 20, 20,
    -10,-20,-20,-20,-20,-20,-20,-10,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
];

pub fn evaluate(move_generator: &MoveGenerator) -> i32 {
    let white_material_eval = count_material(move_generator, Color::White);
    let black_material_eval = count_material(move_generator, Color::Black);
    let (white_positional_eval, black_positional_eval) =
        count_positional_evaluation(move_generator);

    let net_eval = (white_material_eval + white_positional_eval)
        - (black_material_eval + black_positional_eval);

    if move_generator.board.to_move == Color::White {
        net_eval
    } else {
        -net_eval
    }
}

fn count_material(move_generator: &MoveGenerator, color: Color) -> i32 {
    let mut count = 0;
    for square in 0..64 {
        match move_generator.board.colors[square] {
            Some(c) if c == color => {
                count += move_generator.board.squares[square].unwrap().piece_value();
            }
            _ => continue,
        }
    }

    count
}

fn count_positional_evaluation(move_generator: &MoveGenerator) -> (i32, i32) {
    let mut white_count = 0;
    let mut black_count = 0;
    let board = &move_generator.board;

    for square in 0..64 {
        match (board.squares[square], board.colors[square]) {
            (Some(Piece::Pawn), Some(Color::White)) => {
                white_count += WHITE_PAWN_SQUARE_TABLE[square]
            }
            (Some(Piece::Pawn), Some(Color::Black)) => {
                black_count += BLACK_PAWN_SQUARE_TABLE[square]
            }
            (Some(Piece::Knight), Some(Color::White)) => white_count += KNIGHT_SQUARE_TABLE[square],
            (Some(Piece::Knight), Some(Color::Black)) => black_count += KNIGHT_SQUARE_TABLE[square],
            (Some(Piece::Bishop), Some(Color::White)) => {
                white_count += WHITE_BISHOP_SQUARE_TABLE[square]
            }
            (Some(Piece::Bishop), Some(Color::Black)) => {
                black_count += BLACK_BISHOP_SQUARE_TABLE[square]
            }
            (Some(Piece::Rook), Some(Color::White)) => {
                white_count += WHITE_ROOK_SQUARE_TABLE[square]
            }
            (Some(Piece::Rook), Some(Color::Black)) => {
                black_count += BLACK_ROOK_SQUARE_TABLE[square]
            }
            (Some(Piece::Queen), Some(Color::White)) => {
                white_count += WHITE_QUEEN_SQUARE_TABLE[square]
            }
            (Some(Piece::Queen), Some(Color::Black)) => {
                black_count += BLACK_QUEEN_SQUARE_TABLE[square]
            }
            (Some(Piece::King), Some(Color::White)) => {
                white_count += WHITE_KING_MIDDLE_GAME_SQUARE_TABLE[square]
            }
            (Some(Piece::King), Some(Color::Black)) => {
                black_count += BLACK_KING_MIDDLE_GAME_SQUARE_TABLE[square]
            }
            _ => continue,
        }
    }

    (white_count, black_count)
}
