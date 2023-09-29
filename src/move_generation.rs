#![allow(unused)]
use crate::board::Board;
use crate::square::Square;

pub struct Move {
    pub starting_square: Square,
    pub target_square: Square,
}

impl Move {
    pub fn new(starting_square: Square, target_square: Square) -> Self {
        Self {
            starting_square,
            target_square,
        }
    }
}

struct MoveGenerator {
    num_squares_to_edge: [[usize; 8]; 64],
    direction_offsets: [i32; 8],
}

impl Default for MoveGenerator {
    fn default() -> Self {
        MoveGenerator {
            direction_offsets: [8, -8, -1, 1, 7, -7, 9, -9],
            num_squares_to_edge: Self::precompute_move_data(),
        }
    }
}

impl MoveGenerator {
    fn generate_moves(board: Board) -> Vec<Move> {
        let moves: Vec<Move> = Vec::new();
        moves
    }

    fn precompute_move_data() -> [[usize; 8]; 64] {
        let mut num_squares_to_edge = [[0; 8]; 64];
        for file in 0..8 {
            for rank in 0..8 {
                let num_north = 7 - rank;
                let num_south = rank;
                let num_east = 7 - file;
                let num_west = file;

                let square_index = rank * 8 + file;

                num_squares_to_edge[square_index] = [
                    num_north,
                    num_south,
                    num_west,
                    num_east,
                    std::cmp::min(num_north, num_west),
                    std::cmp::min(num_south, num_east),
                    std::cmp::min(num_north, num_east),
                    std::cmp::min(num_south, num_west),
                ];
            }
        }

        num_squares_to_edge
    }
}

#[cfg(test)]
mod tests {
    use crate::move_generation::MoveGenerator;
    use crate::square::Square;

    #[test]
    fn test_num_squares_to_edge() {
        let move_generator: MoveGenerator = MoveGenerator::default();
        // North
        assert_eq!(move_generator.num_squares_to_edge[Square::A1 as usize][0], 7);
        assert_eq!(move_generator.num_squares_to_edge[Square::A4 as usize][0], 4);
        assert_eq!(move_generator.num_squares_to_edge[Square::A8 as usize][0], 0);
        // South
        assert_eq!(move_generator.num_squares_to_edge[Square::A1 as usize][1], 0);
        assert_eq!(move_generator.num_squares_to_edge[Square::A4 as usize][1], 3);
        assert_eq!(move_generator.num_squares_to_edge[Square::A8 as usize][1], 7);
        // West
        assert_eq!(move_generator.num_squares_to_edge[Square::A4 as usize][2], 0);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4 as usize][2], 4);
        assert_eq!(move_generator.num_squares_to_edge[Square::H4 as usize][2], 7);
        // East
        assert_eq!(move_generator.num_squares_to_edge[Square::A4 as usize][3], 7);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4 as usize][3], 3);
        assert_eq!(move_generator.num_squares_to_edge[Square::H4 as usize][3], 0);
        // North West
        assert_eq!(move_generator.num_squares_to_edge[Square::A1 as usize][4], 0);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4 as usize][4], 4);
        assert_eq!(move_generator.num_squares_to_edge[Square::H1 as usize][4], 7);
        // South East
        assert_eq!(move_generator.num_squares_to_edge[Square::A1 as usize][5], 0);
        assert_eq!(move_generator.num_squares_to_edge[Square::A8 as usize][5], 7);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4 as usize][5], 3);
        // North East
        assert_eq!(move_generator.num_squares_to_edge[Square::A1 as usize][6], 7);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4 as usize][6], 3);
        assert_eq!(move_generator.num_squares_to_edge[Square::H4 as usize][6], 0);
        // South West
        assert_eq!(move_generator.num_squares_to_edge[Square::A1 as usize][7], 0);
        assert_eq!(move_generator.num_squares_to_edge[Square::E4 as usize][7], 3);
        assert_eq!(move_generator.num_squares_to_edge[Square::H8 as usize][7], 7);
    }
}
