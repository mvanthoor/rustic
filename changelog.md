# Changelog

## December 28, 2023 - Rustic Alpha 2.3

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used.

- Upgrade 'clap' to 4.4.11
- Upgrade 'crossbeam-channel' to 0.5.10
- Upgrade 'crossbeam-utils' to 0.8.18

## March 28, 2023 - Rustic Alpha 2.2

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used. (Unless you run the engine on the command-line and REALLY want to see
the settings in the About banner. Then you will need to compile a new
version.)

- Update About banner with Hash and Threads settings
- Upgrade 'rand_core' to 0.6.4
- Upgrade 'clap' to 4.1.14
- Upgrade 'crossbeam-channel' to 0.5.7
- Upgrade 'crossbeam-utils' to 0.8.15

## June 11, 2022 - Rustic Alpha 2.1

Maintenance upgrade. There is no functional difference to the previous
version. For normal playing and testing, the existing binaries can be used.

- Upgrade to Rust Edition 2021
- Upgrade 'rand' to 0.8.5
- Upgrade 'rand_chacha' to 0.3.1
- Upgrade 'if_chain' to 1.0.2
- Upgrade 'clap' to 3.2.8
- Upgrade 'crossbeam-channel' to 0.5.5
- Upgrade 'crossbeam-utils' to 0.8.10 (security fix)
- Upgrade 'rand_core' to 0.6.3 (security fix)

## March 17, 2021 - Rustic Alpha 2

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

## December 28, 2023 - Rustic Alpha 1.4

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used.

- Upgrade 'clap' to 4.4.11
- Upgrade 'crossbeam-channel' to 0.5.10
- Upgrade 'crossbeam-utils' to 0.8.18


## March 28, 2023 - Rustic Alpha 1.3

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used.

- Upgrade 'rand_core' to 0.6.4
- Upgrade 'clap' to 4.1.14
- Upgrade 'crossbeam-channel' to 0.5.7
- Upgrade 'crossbeam-utils' to 0.8.15

## June 11, 2021 - Rustic Alpha 1.2

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

## March 15, 2021 - Rustic Alpha 1.1

This is a bugfix release. Alpha 1 lost all of its games on time forfeit
when playing in MoveTime mode (for example, when playing seconds/move).

Bugfixes:
- Do not exceed alotted time in MoveTime mode.

## January 24, 2021 - Rustic Alpha 1

This is the initial release.
Below are the features included in this version.
[CCRL Blitz rating: +/- 1677 Elo](https://www.computerchess.org.uk/ccrl/404/cgi/engine_details.cgi?print=Details&each_game=1&eng=Rustic%20Alpha%201%2064-bit#Rustic_Alpha_1_64-bit)

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