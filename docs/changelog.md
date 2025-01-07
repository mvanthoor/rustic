
<!-- @import "[TOC]" {cmd="toc" depthFrom=1 depthTo=6 orderedList=false} -->

<!-- code_chunk_output -->

- [Changelog](#changelog)
  - [Series 4](#series-4)
    - [Rustic 4.0.0 (TBA)](#rustic-400-tba)
  - [Series Alpha 3](#series-alpha-3)
    - [December 28, 2023 - Rustic Alpha 3.0.4](#december-28-2023---rustic-alpha-304)
    - [March 28, 2023 - Rustic Alpha 3.0.3](#march-28-2023---rustic-alpha-303)
    - [June 11, 2022 - Rustic Alpha 3.0.2](#june-11-2022---rustic-alpha-302)
    - [November 6, 2021 - Rustic Alpha 3.0.1](#november-6-2021---rustic-alpha-301)
    - [June 18, 2021 - Rustic Alpha 3.0.0](#june-18-2021---rustic-alpha-300)
  - [Series Alpha 2](#series-alpha-2)
    - [December 28, 2023 - Rustic Alpha 2.3](#december-28-2023---rustic-alpha-23)
    - [March 28, 2023 - Rustic Alpha 2.2](#march-28-2023---rustic-alpha-22)
    - [June 11, 2022 - Rustic Alpha 2.1](#june-11-2022---rustic-alpha-21)
    - [March 17, 2021 - Rustic Alpha 2](#march-17-2021---rustic-alpha-2)
  - [Series Alpha 1](#series-alpha-1)
    - [December 28, 2023 - Rustic Alpha 1.4](#december-28-2023---rustic-alpha-14)
    - [March 28, 2023 - Rustic Alpha 1.3](#march-28-2023---rustic-alpha-13)
    - [June 11, 2021 - Rustic Alpha 1.2](#june-11-2021---rustic-alpha-12)
    - [March 15, 2021 - Rustic Alpha 1.1](#march-15-2021---rustic-alpha-11)
    - [January 24, 2021 - Rustic Alpha 1](#january-24-2021---rustic-alpha-1)

<!-- /code_chunk_output -->


# Changelog

## Series 4

### Rustic 4.0.0 (TBA)

This version has several new features:

- Tapered and tuned evaluation
- Refactored and cleaned-up code (+75 Elo!)
- Support for the XBoard protocol <sup>(1)</sup>
- Separate Texel tuner
- Split off into executable(s) and library

The engine dropped the "Alpha" part from its name because now everything I
consider to be the basics have been implemented.

- Improvements:
  - TT Clear function: properly clear TT, instead of recreating it.
  - Fix inaccuracy in TT replacement scheme. (+5 Elo for tiny TT's).
  - Fix inaccuracy in TT mate handling (+20 Elo).
  - Drop from 4 to 3 buckets for a bit more speed (+8 Elo).
  - Simplify time management (+25 Elo).
  - pick_move() speed improvement (+3 Elo).
  - Remove unsafe code in move list swap function (0 Elo, +/- 3).
- Refactor:
  - _FINALLY_ fixed the (potential) undefined behavior in MoveList.
  - Restructured Comm to be in line with the rest of the modules.
  - Switch alpha/beta from a strange mix of fail-hard and fail-soft to
    fully fail-soft. No Elo improvement, but the code is cleaner and more
    readable.
  - Better privacy and name spacing for all modules.
  - Made "Entry" the TT index, containing "Buckets" instead of the other
    way around, to be more in line with other engines.
  - Renamed lots of stuff for more consistency.
  - Moved lots of functions between modules for more consistency.
  - Redid nesting for all use-clauses for better readability.
  - Implemented Display for many structs, removing custom functions.
  - Dropped the "misc::print" module (no code left after refactoring).
  - Huge cleanup of FEN-reader error handling code.
- Updated all the libraries
- Dropped crossbeam_channel and crossbeam_utils
- Dropped the extra module: those functions are now separate executables.
- Rustic has been converted into an engine executable + librustic library.
- Fixes
  - Change "-h" / "--hash" command-line options to "-m" / "--memory", to
    avoid conflicts with CLAP's automatic "-h" / "--help" option

> <sup>(1)</sup> Even though the XBoard-protocol was extensively tested,
> the UCI-protocol will remain the default. It is recommended to use UCI
> when testing the engine for rating lists. The XBoard-protocol was
> implemented for completeness, didactic purposes and as an example for
> others who may wish to build a multi-protocol engine.
> 
> If you wish to run Rustic using the XBoard-protocol, you can do so by
> indicating this by appending "**-c xboard**" (without quotes) to the
> engine's startup command. (How this is done depends on the user interface
> you are using.) For example:
> 
> ```
> ./rustic -c xboard
> ```

<hr />

## Series Alpha 3

### December 28, 2023 - Rustic Alpha 3.0.4

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used.

- Upgrade 'clap' to 4.4.11
- Upgrade 'crossbeam-channel' to 0.5.10
- Upgrade 'crossbeam-utils' to 0.8.18

### March 28, 2023 - Rustic Alpha 3.0.3

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used.

- Update About banner layout
- Upgrade 'rand_core' to 0.6.4
- Upgrade 'clap' to 4.1.14
- Upgrade 'crossbeam-channel' to 0.5.7
- Upgrade 'crossbeam-utils' to 0.8.15

### June 11, 2022 - Rustic Alpha 3.0.2

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

### November 6, 2021 - Rustic Alpha 3.0.1

Bugfix upgrade. There is no functional difference to the previous version.
For normal playing and testing, the binary of version 3.0.0 can be used.

- Fixed a variable having the wrong type. This caused the "extra" module
  failing to compile.

### June 18, 2021 - Rustic Alpha 3.0.0

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

<hr />

## Series Alpha 2

### December 28, 2023 - Rustic Alpha 2.3

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used.

- Upgrade 'clap' to 4.4.11
- Upgrade 'crossbeam-channel' to 0.5.10
- Upgrade 'crossbeam-utils' to 0.8.18

### March 28, 2023 - Rustic Alpha 2.2

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

### June 11, 2022 - Rustic Alpha 2.1

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

### March 17, 2021 - Rustic Alpha 2

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
  
<hr />

## Series Alpha 1

### December 28, 2023 - Rustic Alpha 1.4

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used.

- Upgrade 'clap' to 4.4.11
- Upgrade 'crossbeam-channel' to 0.5.10
- Upgrade 'crossbeam-utils' to 0.8.18

### March 28, 2023 - Rustic Alpha 1.3

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used.

- Upgrade 'rand_core' to 0.6.4
- Upgrade 'clap' to 4.1.14
- Upgrade 'crossbeam-channel' to 0.5.7
- Upgrade 'crossbeam-utils' to 0.8.15

### June 11, 2021 - Rustic Alpha 1.2

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

### March 15, 2021 - Rustic Alpha 1.1

This is a bugfix release. Alpha 1 lost all of its games on time forfeit
when playing in MoveTime mode (for example, when playing seconds/move).

Bugfixes:
- Do not exceed alotted time in MoveTime mode.
### January 24, 2021 - Rustic Alpha 1

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