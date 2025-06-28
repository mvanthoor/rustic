# Illegal Move Fix for Sharp Rustic

## Problem Description

The engine was returning `bestmove 0000` (null move) in certain situations, particularly when the search was interrupted early or when the search completed but no principal variation (PV) was found.

## Root Cause Analysis

The issue occurred in the iterative deepening search function (`iter_deep.rs`). The problem was that:

1. The `best_move` was only updated when the search was NOT interrupted (`if !refs.search_info.interrupted()`)
2. If the search was interrupted early, `best_move` remained as the initial null move (`Move::new(0)`)
3. The principal variation (PV) was only built when moves improved alpha, so if the search was interrupted before any move improved alpha, the PV would be empty

## Fix Implementation

### 1. Modified `iter_deep.rs` (lines 85-95)

**Before:**
```rust
if !refs.search_info.interrupted() {
    if !root_pv.is_empty() {
        best_move = root_pv[0];
    }
    // ... rest of the code
}
```

**After:**
```rust
// Always update best_move if we have a valid PV, even if interrupted
if !root_pv.is_empty() {
    best_move = root_pv[0];
} else if !refs.search_info.root_analysis.is_empty() {
    // Fallback: if we have evaluated moves but no PV (interrupted early), 
    // use the first evaluated move as best move
    best_move = refs.search_info.root_analysis[0].mv;
} else if best_move.get_move() == 0 && !refs.search_info.root_analysis.is_empty() {
    // Additional fallback: if best_move is still null but we have root analysis,
    // use the first move from root analysis
    best_move = refs.search_info.root_analysis[0].mv;
}

if !refs.search_info.interrupted() {
    // ... rest of the code
}
```

### 2. Modified `alpha_beta.rs` (lines 360-365)

Added a fallback mechanism to ensure that if we have evaluated moves but no PV (because no move improved alpha), we still set the PV to the best move found:

```rust
// Fallback: if we have evaluated moves but no PV (because no move improved alpha),
// set the PV to the best move found
if is_root && pv.is_empty() && !root_moves.is_empty() {
    pv.push(root_moves[best_index].0);
}
```

## Testing

To test the fix:

1. Build the engine: `cargo build --release`
2. Run the engine: `.\target\release\rustic-sharp.exe`
3. Test with the problematic FEN positions:
   - Starting position: `rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1`
   - Problematic position: `r1bq1rk1/pp3ppp/2nbpn2/3p4/1PP5/P1NBPN2/5PPP/R1BQ1RK1 w - - 0 1`

The engine should now return legal moves instead of `bestmove 0000`.

## Impact

This fix ensures that:
- The engine always returns a valid legal move when one exists
- The search can be interrupted safely without losing the best move found
- The engine is more robust against edge cases in search termination 