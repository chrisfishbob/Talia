use reqwest::{self, blocking::Client};
use serde::Deserialize;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::{
    errors::BoardError,
    evaluate::evaluate,
    move_generation::{Flag, Move, MoveGenerator},
};

const INF: i32 = i32::MAX;
pub static COUNTER: AtomicI32 = AtomicI32::new(0);

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct TablebaseResponse {
    pub dtz: Option<i32>,
    pub precise_dtz: Option<i32>,
    pub dtm: Option<i32>,
    pub checkmate: bool,
    pub stalemate: bool,
    pub insufficient_material: bool,
    pub category: Category,
    pub moves: Vec<TablebaseMove>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct TablebaseMove {
    uci: String,
    san: String,
    dtz: Option<i32>,
    precise_dtz: Option<i32>,
    dtm: Option<i32>,
    zeroing: bool,
    checkmate: bool,
    stalemate: bool,
    insufficient_material: bool,
    category: Category,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub enum Category {
    #[serde(rename = "win")]
    Win,
    #[serde(rename = "loss")]
    Loss,
    #[serde(rename = "draw")]
    Draw,
}

impl TablebaseResponse {
    fn get_best_move(&self) -> &TablebaseMove {
        let mut best_move = &self.moves[0];

        for mv in &self.moves {
            match best_move.category {
                Category::Win => {
                    if mv.category == Category::Draw || mv.category == Category::Loss {
                        best_move = mv
                    }
                }
                Category::Loss => {
                    if mv.category == Category::Loss && mv.dtm > best_move.dtm {
                        best_move = mv
                    }
                }
                Category::Draw => {
                    if mv.category == Category::Loss {
                        best_move = mv
                    }
                }
            }
        }

        best_move
    }
}

pub fn search(move_generator: &mut MoveGenerator, depth: u32, mut alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        COUNTER.fetch_add(1, Ordering::Relaxed);
        return search_all_captures(move_generator, alpha, beta);
    }

    let mut moves = move_generator.generate_moves();
    if moves.is_empty() {
        if move_generator.is_in_check(move_generator.board.to_move) {
            return -INF;
        } else {
            return 0;
        }
    }

    moves.sort_unstable_by_key(|mv| guess_move_score(move_generator, mv));
    for mv in moves.iter() {
        move_generator.board.move_piece(mv);
        let eval = -search(move_generator, depth - 1, -beta, -alpha);
        move_generator.board.unmake_move(mv).unwrap();

        if eval >= beta {
            // Move too good, opponent will avoid
            return beta;
        }

        alpha = std::cmp::max(eval, alpha);
    }

    alpha
}

// TODO: Modify move generation to make this more efficient
fn search_all_captures(move_generator: &mut MoveGenerator, alpha: i32, beta: i32) -> i32 {
    let eval = evaluate(move_generator);
    if eval >= beta {
        return beta;
    }

    let mut alpha = std::cmp::max(alpha, eval);
    let mut capture_moves: Vec<Move> = move_generator
        .generate_moves()
        .into_iter()
        .filter(|mv| {
            matches!(
                mv.flag,
                Flag::EnPassantCapture | Flag::Capture(_) | Flag::CaptureWithPromotion(_, _)
            )
        })
        .collect();
    capture_moves.sort_unstable_by_key(|mv| guess_move_score(move_generator, mv));

    for mv in capture_moves.iter() {
        move_generator.board.move_piece(mv);
        let eval = -search_all_captures(move_generator, -beta, -alpha);
        move_generator.board.unmake_move(mv).unwrap();

        if eval >= beta {
            return beta;
        }
        alpha = std::cmp::max(alpha, eval)
    }

    alpha
}

// TODO: Use standard query rather than mainline
pub fn query_tablebase(move_generator: &mut MoveGenerator) -> Result<(Move, i32), BoardError> {
    let base_tb_server_url = "http://tablebase.lichess.ovh/standard";
    // Make FEN URL friendly
    let params = [("fen", move_generator.board.to_fen().replace(' ', "_"))];
    let client = Client::new();
    let response = client
        .get(base_tb_server_url)
        .query(&params)
        .send()
        .map_err(|err| BoardError::new(&format!("{err}")))?;

    let tb_response: TablebaseResponse = if response.status().is_success() {
        response
            .json()
            .map_err(|err| BoardError::new(&format!("{err}")))?
    } else {
        return Err(BoardError::new("Call to tablebase failed"));
    };

    let best_move = tb_response.get_best_move();
    let eval = match best_move.category {
        Category::Win => -INF,
        Category::Draw => 0,
        Category::Loss => INF,
    };

    Ok((
        Move::try_from_algebraic_notation(&best_move.uci, move_generator)?,
        eval,
    ))
}

pub fn find_best_move(
    moves: &mut [Move],
    move_generator: &mut MoveGenerator,
    depth: u32,
) -> (Move, i32) {
    COUNTER.store(0, Ordering::Relaxed);

    let pieces_left = move_generator
        .board
        .squares
        .iter()
        .filter(|sq| sq.is_some())
        .count();
    if pieces_left <= 7 {
        // TODO: Add logging for when query fails
        match query_tablebase(move_generator) {
            Ok(tb_result) => return tb_result,
            Err(err) => println!("{err}"),
        }
    }
    moves.sort_unstable_by_key(|mv| guess_move_score(move_generator, mv));

    let mut best_move = moves
        .get(0)
        .expect("moves vector must have at least one move");

    let mut alpha = -INF;
    let beta = INF;

    for mv in moves.iter() {
        move_generator.board.move_piece(mv);
        let eval = -search(move_generator, depth - 1, -beta, -alpha);
        move_generator.board.unmake_move(mv).unwrap();
        if eval > alpha {
            alpha = eval;
            best_move = mv;
        }
    }

    (best_move.clone(), alpha)
}

pub fn guess_move_score(move_generator: &MoveGenerator, mv: &Move) -> i32 {
    let mut score_guess: i32 = 0;

    let starting_piece = move_generator.board.squares[mv.starting_square].unwrap();

    match mv.flag {
        Flag::PromoteTo(piece) => score_guess += piece.piece_value(),
        Flag::Capture(piece) => {
            score_guess += 10 * piece.piece_value() - starting_piece.piece_value()
        }
        Flag::CaptureWithPromotion(captured_piece, promotion_piece) => {
            score_guess += promotion_piece.piece_value() + 10 * captured_piece.piece_value()
                - starting_piece.piece_value()
        }
        _ => (),
    }

    // Negate score so that the moves with the highest score will be first
    -score_guess
}

#[cfg(test)]
mod tests {
    use crate::{
        board::Board,
        board_builder::BoardBuilder,
        errors::BoardError,
        move_generation::{Flag, Move, MoveGenerator},
        piece::{Color, Piece},
        search::INF,
        square::Square,
    };

    use super::find_best_move;

    #[test]
    fn test_find_best_move_mate_in_one() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::H3, Piece::King, Color::Black)
            .piece(Square::A8, Piece::Rook, Color::Black)
            .to_move(Color::Black)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = move_generator.generate_moves();
        let (best_move, eval) = find_best_move(&mut moves, &mut move_generator, 2);
        let mating_move = Move::from_square(Square::A8, Square::A1, Flag::None);

        assert!(best_move == mating_move);
        assert!(eval == INF);

        Ok(())
    }

    #[test]
    fn test_captures_handing_queen() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::new()
            .piece(Square::H1, Piece::King, Color::White)
            .piece(Square::A8, Piece::King, Color::Black)
            .piece(Square::E1, Piece::Rook, Color::White)
            .piece(Square::E5, Piece::Queen, Color::Black)
            .to_move(Color::White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = move_generator.generate_moves();
        let (best_move, _) = find_best_move(&mut moves, &mut move_generator, 2);
        let capture_move = Move::from_square(Square::E1, Square::E5, Flag::Capture(Piece::Queen));

        assert!(best_move == capture_move);

        Ok(())
    }

    #[test]
    fn test_pins_queen_to_king() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::new()
            .piece(Square::F1, Piece::King, Color::White)
            .piece(Square::E8, Piece::King, Color::Black)
            .piece(Square::E5, Piece::Queen, Color::Black)
            .piece(Square::A1, Piece::Rook, Color::White)
            .to_move(Color::White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = move_generator.generate_moves();
        let (best_move, _) = find_best_move(&mut moves, &mut move_generator, 3);
        let capture_move = Move::from_square(Square::A1, Square::E1, Flag::None);

        assert!(best_move == capture_move);

        Ok(())
    }

    #[test]
    fn test_forks_king_and_queen() -> Result<(), BoardError> {
        let board: Board = BoardBuilder::new()
            .piece(Square::F1, Piece::King, Color::White)
            .piece(Square::C4, Piece::King, Color::Black)
            .piece(Square::G4, Piece::Queen, Color::Black)
            .piece(Square::D1, Piece::Knight, Color::White)
            .to_move(Color::White)
            .try_into()?;

        let mut move_generator = MoveGenerator::new(board);
        let mut moves = move_generator.generate_moves();
        let (best_move, _) = find_best_move(&mut moves, &mut move_generator, 3);
        let forking_move = Move::from_square(Square::D1, Square::E3, Flag::None);

        assert!(best_move == forking_move);

        Ok(())
    }
}
