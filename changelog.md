# Changelog

## ?? - Rustic Alpha 2

- Features:
  - Transposition table implemented for perft
  - -h / --hash added to command line for setting hash size
  - Ability to announce and set engine options (uci)
- Bugfixes:
  - Move check extension higher up in the search routine, to prevent
    quiescence search while in check.
- Changes:
  - seldepth: report max ply reached, instead of selective depth at last
    completed iteration.
  - Transition random number generators from SmallRng to ChaChaRng for
    reproducible behavior between platforms/OS's/architectures/versions.
- Cleanup
  - Change Root PV handling to be able to remove redundant code.
  - Miscellaneous small renames, refactors, and cleanups.
  - Add rand_chacha and remove SmallRng number generators.
  - Update Rand library to 0.8.3.

## January 24, 2021 - Rustic Alpha 1

This is the initial release.
Below are the features included in this version.
[CCRL Blitz rating: +/- 1695 Elo](https://www.computerchess.org.uk/ccrl/404/cgi/engine_details.cgi?print=Details&each_game=1&eng=Rustic%20Alpha%201%2064-bit#Rustic_Alpha_1_64-bit)

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