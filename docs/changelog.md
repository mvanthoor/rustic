
<!-- @import "[TOC]" {cmd="toc" depthFrom=1 depthTo=6 orderedList=false} -->

<!-- code_chunk_output -->

- [Changelog](#changelog)
  - [Rustic 4.0.0 (2021, TBA)](#rustic-400-2021-tba)
  - [Rustic Alpha 3.0.0 (2021, June 18)](#rustic-alpha-300-2021-june-18)
  - [Rustic Alpha 2 (2021, March 17)](#rustic-alpha-2-2021-march-17)
  - [Rustic Alpha 1.1 (2021, March 15)](#rustic-alpha-11-2021-march-15)
  - [Rustic Alpha 1 (2021, January 24)](#rustic-alpha-1-2021-january-24)

<!-- /code_chunk_output -->
# Changelog

## Rustic 4.0.0 (2021, TBA)

- New features:
  - Tapered and tuned evaluation.
- Improvements:
  - TT Clear function: properly clear TT, instead of recreating it.
  - Fix inaccuracy in TT replacement scheme. (+5 Elo for tiny TT's).
  - Drop from 4 to 3 buckets for a bit more speed (+8 Elo).
  - Simplify time management (+30 Elo in self-play).
- Misc:
  - Refactor: Switch alpha/beta from a strange mix of fail-hard and
    fail-soft to fully fail-soft. No Elo improvement, but the code is
    cleaner and more readable.
  - Refactor: Better privacy and namespacing for several modules.
  - Cleanup: Made "Entry" the TT index, containing "Buckets" instead of
    the other way around, to be more in line with other engines.
  - Cleanup: Renamed some variables here and there for consistency.
  - Update "rand" crate to 0.8.4.
  - Update "rand_chacha" crate to 0.3.1.
  - Update "if_chain" crate to 1.0.2.

## Rustic Alpha 3.0.0 (2021, June 18)

[CCRL Blitz rating: +/- 1867 Elo](https://www.computerchess.org.uk/ccrl/404/cgi/engine_details.cgi?print=Details&each_game=1&eng=Rustic%20Alpha%203.0.0%2064-bit#Rustic_Alpha_3_0_0_64-bit)

- New features:
  - Killer Moves
  - Principal Variation Search (PVS)
- Changes:
  - Switch versioning scheme to SemVer. Versions are going to be in the
    form "a.b.c" from now on, with the following meaning:
    - Increment **a**: A new strength-gaining feature was added.
    - Increment **b**: A bug was fixed that gained strength.
    - Increment **c**: A feature was added or a bug was fixed that did not
      gain stregnth. It is not necessary to test this version for a rating
      change.
- Misc:
  - Updated crossbeam-channel to version 0.5.1
  - A Makefile was added, so Rustic can be built using "GNU Make". When
    typing "make" (or "gmake" in MacOS), the Makefile will build all Rustic
    versions for the platform it's being compiled on.
  - Re-add showing the size of the TT and number of threads in About.
  - Fairly large update of the book on https://rustic-chess.org/.

## Rustic Alpha 2 (2021, March 17)

[CCRL Blitz rating: +/- 1815 Elo](https://ccrl.chessdom.com/ccrl/404/cgi/engine_details.cgi?print=Details&each_game=1&eng=Rustic%20Alpha%202%2064-bit#Rustic_Alpha_2_64-bit)

- New Features:
  - Transposition table for search and perft.
  - Ordering on transposition table move.
  - Set TT size through --hash option or UCI parameter.
- Improvement:
  - Move check extension higher up in the search routine, to prevent
    quiescence search while in check.
- Changes:
  - seldepth: report max ply reached during the search, instead of
    selective depth at last completed iteration.
  - Count all nodes visited, instead of only nodes which generated moves.
  - Change random number generator from SmallRng to ChaChaRng for
    reproducible behavior between platforms/OS's/architectures/versions.
- Cleanup
  - Change Root PV handling to remove redundant code.
  - Miscellaneous small renames, refactors, and cleanups.
  - Add rand_chacha and remove SmallRng number generators.
  - Update Rand library to 0.8.3.

## Rustic Alpha 1.1 (2021, March 15)

This is a bugfix release. Alpha 1 lost all of its games on time forfeit
when playing in MoveTime mode (for example, when playing seconds/move).

Bugfixes:
- Do not exceed alotted time in MoveTime mode.

## Rustic Alpha 1 (2021, January 24)

This is the initial release.

[CCRL Blitz rating: +/- 1677 Elo](https://www.computerchess.org.uk/ccrl/404/cgi/engine_details.cgi?print=Details&each_game=1&eng=Rustic%20Alpha%201%2064-bit#Rustic_Alpha_1_64-bit)

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