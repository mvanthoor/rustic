# Changelog

## ?? - Rustic Alpha 2

- Bugfixes:
  - Move check extension higher up in the search routine, to prevent
    quiescence search while in check.
- Changes:
  - seldepth: report max ply reached (like Stockfish).
- Cleanup
  - Change Root PV handling and remove redundant code.
  - Update Rand library to 0.8.3.
  - Miscellaneous small renames and cleanups.

## January 24, 2021 - Rustic Alpha 1

This is the initial release.
Below are the features included in this version.

- Engine:
  - Bitboard board representation
  - Magic bitboard move generator
  - UCI-protocol
- Search
  - Alpha/Beta search
  - Quiescence search
  - MVV-LVA move ordering
  - Check extension
- Evaluation
  - Material counting
  - Piece-Square Tables