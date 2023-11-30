# Talia

## Overview
Talia is a UCI-compliant, thoroughly tested chess engine written from the ground up in Rust.


## Instructions
### Terminal
To play a game against Talia in the terminal, simply run `cargo run --release`.  
***!Do not forget the release flag!***  
Moves can be inputted via UCI notation, which is simply the start square immediately followed
by the target square.
```
Talia Chess Engine: v1.1.0

8  ['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r']

7  ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p']

6  [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ']

5  [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ']

4  [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ']

3  [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ']

2  ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P']

1  ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R']

     A    B    C    D    E    F    G    H

White to move.

e2e4

Talia is thinking ...
Talia thought for 604 milliseconds and evaluted 77218 positions at depth 6
Best move: starting_square: B8, target_square: C6
Eval: 35

8  ['r', ' ', 'b', 'q', 'k', 'b', 'n', 'r']

7  ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p']

6  [' ', ' ', 'n', ' ', ' ', ' ', ' ', ' ']

5  [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ']

4  [' ', ' ', ' ', ' ', 'P', ' ', ' ', ' ']

3  [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ']

2  ['P', 'P', 'P', 'P', ' ', 'P', 'P', 'P']

1  ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R']

     A    B    C    D    E    F    G    H

White to move.

```

### GUI
Interop with Chess GUIs and Lichess via the UCI protocol is underway and will be available soon.

## Progress

### Legal Move Generation Complete
✅ Piece representation  
✅ Board representation  
✅ Converting a FEN string to the internal board representation  
✅ Exporting the internal board representation to a FEN string  
✅ Board builder  
✅ Queen move generation  
✅ Rook move generation  
✅ Bishop move generation  
✅ Pawn single move generation   
✅ Pawn double move generation  
✅ Pawn promotions move generation  
✅ Pawn captures moves generation  
✅ Pawn en passant captures moves generation  
✅ King single moves generation   
✅ Kingside castling move generation  
✅ Queenside castling move generation  

### Search && Evaluation
✅ Minimax Search  
✅ Alpha-beta pruning  
✅ Move ordering heuristics  
✅ Position evaluation through material  
✅ Quiescence search  
✅ 7 piece Endgame syzygy tablebase  
✅ Position evaluation through piece square tables 

## Playing Strength
Currently, Talia is around 1800+ ELO on Lichess and is improving rapidly.  
I'll probably never beat her ever again, YMMV.
