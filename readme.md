![Rustic Banner](https://rustic-chess.org/img/rustic-logo-web.jpg)

[https://rustic-chess.org/](https://rustic-chess.org/)

# Rustic Chess Engine

Rustic is a chess engine written from scratch in the Rust programming
language. It is not derived from any other engine but it uses many concepts
that have been well-known in chess programming for many decades; therefore,
the engine still stands on the shoulders of giants.

>__NOTICE__: This is Rustic's master branch. It contains the latest
development on the engine, which is currently _4.0-beta_. It can sometimes
have big changes merged into it. If you are looking for a version that will
never change except for maintenance updates such as crate and edition
bumps, you can take a look at the older Alpha 3.x, 2.x and 1.x series. I'll
gladly accept pull requests with targeted fixes, tweaks, or small
improvements. However, there have been PR's which add completely new
features or do large refactorings. If you intend to develop your own chess
engine with any version of Rustic as a base, then fork it and go for it!
That's what the engine is intended for. However, I'm not going to accept
very large PR's that will add large features or do significant
refactorings.

# User interface

The engine does not provide its own user interface. It uses the UCI and
XBoard protocols to communicate with graphical user interfaces. (XBoard is
not yet completely implemented at the time of writing. For now, use the
engine in UCI mode, which is also the default.) It is recommended that you
use a GUI to play games against the engine. Rustic is tested with these
user interfaces:

- [Arena Chess GUI](http://www.playwitharena.de/)
- [XBoard/Winboard](https://www.gnu.org/software/xboard/FAQ.html)
- [CuteChess](https://cutechess.com/)
- [Tarrasch](https://www.triplehappy.com/)
- [The Shredder GUI](https://www.shredderchess.com/)
- [Fritz / Chessbase series](https://en.chessbase.com/)
- [Scid vs PC (database)](http://scidvspc.sourceforge.net/)
- [Banksia GUI](https://banksiagui.com/)

There are many other user interfaces that will probably work just fine, but its
obviously impossible to test all of them. If you have problems with a user
interface, open an issue so I can see if it can be fixed. (Assuming the
user interface is either free or open source, as I can't go and buy GUI's
just for testing purposes.)

# Features

The current feature-set for Rustic Alpha 3.0.0 is:

- Engine:
  - Bitboard board representation
  - Fancy Magic bitboard move generator
  - Transposition Table
  - UCI-protocol
- Search
  - Alpha/Beta search
  - Quiescence search
  - Check extension
  - PVS
- Move ordering
  - TT Move priority
  - MVV-LVA
  - Killer moves
- Evaluation
  - Material counting
  - Piece-Square Tables

(See changelog.md for more information.)

# Included binaries, supported platforms

Each release contains several binaries, optimized for different CPU's and
platforms. Note that I'll only create binaries for the most-used CPU's and
operating systems, but if Rust is available on your system, you can
probably compile the engine yourself without needing to change the code.

Windows still supports a 32-bit executable. Note that this executable is at
50% slower (half the speed) as compared to the 64-bit one and not as well
tested. Many Linux-distributions have dropped support for 32-bit native
Linux software, so no 32-bit executable for Linux is provided. The same is
true for MacOS. The executable for the Raspberry Pi will be 32-bit as long
as the 32-bit version of Raspbian OS is the default and 64-bit is
experimental.

You can use the binary which runs fastest on your particular system for
maximum playing strength. Start a terminal, and run each Rustic version:

```
$ ./<executable_name> -p7 -m512
```

This will run perft 7 from the starting position, using a 512 MB
transposition table. Pick the version that runs perft 7 the fastest. If a
binary crashes, your CPU does not support the required instructions to run
it. Try a different binary.

If you wish to run Rustic on a system for which no binary is supplied, you
can try to compile the engine yourself using the compilation tips below.
Make sure to install at least Rust version 1.46.

# Building Rustic

There is a [dedicated page](./docs/build.md) which explains how you can
build Rustic for your platform if you wish to create your own binaries, or
want to run tests by using your own compiler flags.

# All command-line options

```
USAGE:
    rustic.exe [FLAGS] [OPTIONS]

FLAGS:
        --help        Prints help information
    -k, --kiwipete    Set up KiwiPete position (ignore --fen)
    -q, --quiet       No intermediate search stats updates
    -V, --version     Prints version information

OPTIONS:
    -c, --comm <comm>          Select communication protocol to use [default: uci]  [possible values: uci, xboard]
    -f, --fen <fen>            Set up the given position [default: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -
                               0 1]
    -h, --hash <hash>          Transposition Table size in MB [default: 32]
    -p, --perft <perft>        Run perft to the given depth [default: 0]
    -t, --threads <threads>    Number of CPU-threads to use [default: 1]
```

Please note that the -e (--epdtest) and -w (--wizardry) options are only
available if the "extra" module is compiled into the engine.

# Credits

More extensive credits can be found in "credits.md", or in [Rustic's
documentation](https://rustic-chess.org/back_matter/credits.html).

Many people have assisted in one way or another, in the development of
Rustic. They are listed below, in no particular order.

> My girlfriend (for providing lots of support in more ways than she'd ever
> be able to realize), Richard Allbert, Terje Kirstihagen, Fabian von der
> Warth, H.G. Müller, Maksim Korzh, Rasmus Althoff, Martin Sedlák, Ronald
> de Man, Taimo, Sven Schüle, Thomas Jahn, Christian Dean, Brandon Ros, and
> last but not least, Ed Schröder and Robert Hyatt (for still hanging
> around the chess programming community after 40 or 50 years).

All of these people (except my girlfriend) can be found on the
[Talkchess.com](http://talkchess.com/forum3/index.php) forum.

