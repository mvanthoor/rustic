# Enhanced Time Management System

This document describes the enhanced time management system implemented in Sharp Rustic chess engine.

## Overview

The enhanced time management system provides three main improvements:

1. **Emergency Time Management** - Prevents time losses in critical situations
2. **Adaptive Moves-to-Go** - Dynamically estimates remaining moves based on game phase
3. **Statistics Tracking** - Monitors time usage patterns for continuous improvement

## Features

### 1. Emergency Time Management

When the engine has less than 2 seconds per move available, it automatically enters emergency mode:

- **Depth Limitation**: Maximum search depth is reduced to 8 plies
- **Time Reduction**: Uses only 50% of normal allocated time
- **Automatic Detection**: Triggers when `clock < moves_to_go * 2000ms`

```rust
// Emergency mode thresholds
const EMERGENCY_TIME_THRESHOLD: u128 = 2_000; // 2 seconds per move
const EMERGENCY_MAX_DEPTH: i8 = 8; // Maximum depth in emergency mode
const EMERGENCY_TIME_FACTOR: f64 = 0.5; // Use 50% of normal time
```

### 2. Adaptive Moves-to-Go Estimation

Instead of using a fixed 25-move estimation, the engine now adapts based on game phase:

| Game Phase | Ply Range | Piece Count | Estimated Moves |
|------------|-----------|-------------|-----------------|
| Opening | 0-20 | Any | 30 |
| Early Middlegame | 21-30 | ≥20 | 25 |
| Early Middlegame | 21-30 | 10-19 | 20 |
| Late Middlegame | 31-40 | ≥10 | 15 |
| Endgame | >40 or ≤12 pieces | Any | 10 |

### 3. Game Phase Detection

The engine automatically detects the current game phase:

- **Opening**: First 20 moves
- **Early Middlegame**: Moves 21-30
- **Late Middlegame**: Moves 31-40
- **Endgame**: After move 40 or when ≤12 pieces remain

### 4. Time Control Classification

The engine classifies time controls and adjusts strategy accordingly:

- **Bullet**: < 3 minutes (80% of normal time)
- **Blitz**: 3-15 minutes (90% of normal time)
- **Rapid**: 15-60 minutes (100% of normal time)
- **Classical**: > 60 minutes (110% of normal time)

### 5. Move Quality Assessment

Time allocation is adjusted based on position complexity:

- **Excellent**: Clear best move (70% of normal time)
- **Good**: Good move with alternatives (85% of normal time)
- **Acceptable**: Multiple reasonable moves (100% of normal time)
- **Poor**: Difficult position (120% of normal time)
- **Critical**: Critical position, in check (150% of normal time)

### 6. Statistics Tracking

The engine tracks time usage statistics:

- Total moves played
- Successful time allocations
- Time losses
- Average time per move
- Phase-specific time usage
- Success rate percentage

## Implementation Details

### Key Functions

1. **`emergency_time_management()`** - Detects and activates emergency mode
2. **`adaptive_moves_to_go()`** - Estimates remaining moves based on game phase
3. **`determine_game_phase()`** - Identifies current game phase
4. **`classify_time_control()`** - Categorizes time control type
5. **`assess_move_quality()`** - Evaluates position complexity
6. **`calculate_enhanced_time_slice()`** - Combines all factors for final time allocation
7. **`update_time_statistics()`** - Records time usage data

### Integration Points

- **Iterative Deepening**: Emergency mode and enhanced time calculation
- **Alpha-Beta Search**: Time checking with emergency mode limits
- **GUI Communication**: Statistics reporting for monitoring

## Usage

The enhanced time management is automatically active when using game time mode. Statistics are reported to the GUI after each move for monitoring purposes.

### Example Output

```
info string Time Stats: Total=15, Success=14, Rate=93.3%, Avg=1250ms, Phase=EarlyMiddlegame, Control=Blitz, Emergency=false
```

## Benefits

1. **Prevents Time Losses**: Emergency mode prevents flagging in critical situations
2. **Better Time Distribution**: Adaptive estimation provides more accurate time allocation
3. **Phase-Aware Strategy**: Different strategies for different game phases
4. **Continuous Improvement**: Statistics tracking enables performance monitoring
5. **Robust Performance**: Multiple fallback mechanisms ensure reliable operation

## Future Enhancements

Potential improvements for future versions:

1. **Machine Learning**: Use statistics to learn optimal time allocation patterns
2. **Opponent Modelling**: Adjust strategy based on opponent's time management
3. **Position Complexity**: More sophisticated position evaluation for time allocation
4. **Multi-PV Time Management**: Consider multiple principal variations in time allocation
5. **Historical Analysis**: Use game history to improve moves-to-go estimation 