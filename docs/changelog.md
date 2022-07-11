
<!-- @import "[TOC]" {cmd="toc" depthFrom=1 depthTo=6 orderedList=false} -->

<!-- code_chunk_output -->

- [Changelog](#changelog)
  - [Rustic Alpha 3.0.1 (2021, November 6)](#rustic-alpha-301-2021-november-6)
  - [Rustic Alpha 3.0.0 (2021, June 18)](#rustic-alpha-300-2021-june-18)
  - [Rustic Alpha 2 (2021, March 17)](#rustic-alpha-2-2021-march-17)
  - [Rustic Alpha 1.1 (2021, March 15)](#rustic-alpha-11-2021-march-15)
  - [Rustic Alpha 1 (2021, January 24)](#rustic-alpha-1-2021-january-24)

<!-- /code_chunk_output -->
# Changelog

## Rustic Alpha 3.0.2 / 2.1 / 1.2 (2022, June 11)

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used.

- Upgrade to Rust Edition 2021
- Upgrade 'rand' to 0.8.5
- Upgrade 'rand_chacha' to 0.3.1
- Upgrade 'if_chain' to 1.0.2
- Upgrade 'clap' to 3.2.8
- Upgrade 'crossbeam-channel' to 0.5.5
- Upgrade 'crossbeam-utils' to 0.8.10 (security fix)
- Upgrade 'rand_core' to 0.6.3 (security fix)

## Rustic Alpha 3.0.1 (2021, November 6)

Bugfix upgrade. There is no functional difference to the previous version.
For normal playing and testing, the binary of version 3.0.0 can be used.

- Fixed a variable having the wrong type. This caused the "extra" module
  failing to compile.

## Rustic Alpha 3.0.0 (2021, June 18)

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