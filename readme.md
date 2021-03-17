![Rustic Banner](https://rustic-chess.org/img/rustic-logo-web.jpg)

[https://rustic-chess.org/](https://rustic-chess.org/)

# Rustic Chess Engine

Rustic is a chess engine written from scratch in the Rust programming
language. It is not derived from any other engine, but it uses many
concepts that have been well-known in chess programming for many decades;
therefore, the engine still stands on the shoulders of the giants of the
past.

# User interface

The engine does not provide its own user interface. It uses the UCI and
XBoard protocols to communicate with graphical user interfaces. (XBoard is
not yet completely implemented at the time of writing. For now, use the
engine in UCI mode, which is the default.) It is recommended that you use
such an interface to play games against the engine. Rustic is tested with
these user interfaces:

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

At the time of writing (15-03-2020) Rustic does not have many features yet,
but the basics for playing decent chess have been implemented. Obviously,
features will be incrementally added, so the approximate strength gain per
feature can be determined. (See changelog.md for more information.)

The current feature-set for Rustic Alpha 2 is:

- Engine:
  - Bitboard board representation
  - Magic bitboard move generator
  - Transposition Table
  - UCI-protocol
- Search
  - Alpha/Beta search
  - Quiescence search
  - Check extension
- Move ordering
  - TT Move priority
  - MVV-LVA
- Evaluation
  - Material counting
  - Piece-Square Tables

There are many features that will be added in the future, such as:
- Finishing the XBoard protocol.
- Several pruning options in the search
- Tapered evaluation for middle game/endgame
- Many more evaluation terms
- Add Lazy SMP

These will be listed under "Features" as they are implemented.

# Included binaries, supported platforms

There are several binaries supplied in the Rustic Alpha 1 release. For
Windows, a generic 32-bit binary is supplied. As most Linux distributions
have dropped 32-bit support (or are in the process of doing so), only
64-binaries are included. As long as the Raspberry Pi OS is not yet
officially 64-bit, only a 32-bit version that runs on the Buster release is
supplied.

The Windows binaries have been tested on Windows 10,
but will probably also work on Windows 8.x or 7, as long as the correct C++
Redistributables are installed.

The Linux binaries have been created on Debian 8 Stable ("Jessie"), and
tested on Debian 9 and 10 Stable. They should run on any Debian-based
installateion that has the library versions of Debian 8 stable or newer
installed. To my regret I don't have the time or the resources to provide
lots of binaries for other versions of Linux; I only ever use Debian
Stable. If you wish to run Rustic on a different distribution (if it
doesn't do so out of the box), then try and compile it yourself using the
compilation tips below.

- Windows (tested on Windows 10)
  - 32-bit generic
  - 64-bit old
  - 64-bit popcnt
  - 64-bit bmi2
- Linux (since Debian 8 stable)
  - 64-bit old
  - 64-bit popcnt
  - 64-bit bmi2
- Raspberry Pi, Buster
  - 32-bit

# Quick compiling tips

Follow the instructions below if you want to compile the engine yourself.

- First install [Rust](https://www.rust-lang.org/) for your platform. Make
  sure you install at least version 1.46, as Rustic uses featurse that only
  became available in that version.
- Make sure you install the correct toolchain for your platform.
  - Linux:
    - stable-x86_64-unknown-linux-gnu
  - Windows:
    - stable-x86_64-pc-windows-gnu (This toolchain creates compiles that
      are compatible with the GNU GDB-debugger.)
    - stable-x86_64-pc-windows-msvc (This toolchain creates compiles that
      are compatible with Windows/Visual Studio's debugger. This will
      require the Microsoft Visual C++ Build Tools, because it uses the
      Microsoft Linker.)
- If you are running Windows, it is recommended to have
  [MSYS2](https://www.msys2.org/) installed, because it provides Bash and
  several Linux development tools.
  There are three ways to run MSYS:
    1. MSYS2 MinGW64: for 64-bit compiles
    2. MSYS2 MinGW32: for 32-bit compiles
    3. MSYS2 MSYS: for maintaining MSYS2.
- Install GCC
- Install Binutils.
- Make sure you keep these environments apart. Do not install the 64-bit
  GCC compiler in the 32-bit environment and the other way around. If you
  want to produce both 32-bit and 64-bit binaries, you will have to set up
  both the MSYS2 MinGW32 and MSYS2 MinGW64 command lines. Again: only
  install the utils/compilers prefixed with MinGW32/ on MSYS2 MinGW32, and
  the ones prefixed with MinGW64/ on MSYS2 MinGW64. If you don't, you
  *will* get conflicts.
- Git clone (or download/unzip) the Rustic repository onto your computer.
- Change to the root folder of the repository (where cargo.toml is).
- Start either Bash in Linux, or MSYS2 on Windows, and switch to the root
  folder of the repository. Now you can build for either Linux or Windows:

**Linux**

```
Create the bin folder:

mkdir -p ./bin/linux

64-bit old (Core2 CPU's and newer):

rm -rf ./target && \
export RUSTFLAGS="-C target-cpu=core2" && \
cargo build --release && \
strip -s ./target/release/rustic && \
mv ./target/release/rustic ./bin/linux/rustic-alpha-2_64-bit-old
	
64-bit popcnt (Nehalem CPU's and newer):

rm -rf ./target && \
export RUSTFLAGS="-C target-cpu=nehalem" && \
cargo build --release && \
strip -s ./target/release/rustic && \
mv ./target/release/rustic ./bin/linux/rustic-alpha-2_64-bit-popcnt

64-bit bmi2 (Haswell CPU's and newer):

rm -rf ./target && \
export RUSTFLAGS="-C target-cpu=haswell" && \
cargo build --release && \
strip -s ./target/release/rustic && \
mv ./target/release/rustic ./bin/linux/rustic-alpha-2_64-bit-bmi2

64-bit native (Compiles for your current CPU):

rm -rf ./target && \
export RUSTFLAGS="-C target-cpu=native" && \
cargo build --release && \
strip -s ./target/release/rustic && \
mv ./target/release/rustic ./bin/linux/rustic-alpha-2_64-bit-native
```

**Windows**

```
Create the bin folder:

mkdir -p ./bin/windows

32-bit Generic (Should run on anything since the Pentium II):

rm -rf ./target && \
cargo build --release --target="i686-pc-windows-gnu" && \
strip -s ./target/i686-pc-windows-gnu/release/rustic.exe && \
mv ./target/i686-pc-windows-gnu/release/rustic.exe ./bin/windows/rustic-alpha-2_32-bit-generic.exe

64-bit old (Core2 CPU or newer):

rm -rf ./target && \
export RUSTFLAGS="-C target-cpu=core2" && \
cargo build --release && \
strip -s ./target/release/rustic.exe && \
mv ./target/release/rustic.exe ./bin/windows/rustic-alpha-2_64-bit-old.exe
	
64-bit popcnt (Nehalem CPU or newer):

rm -rf ./target && \
export RUSTFLAGS="-C target-cpu=nehalem" && \
cargo build --release && \
strip -s ./target/release/rustic.exe && \
mv ./target/release/rustic.exe ./bin/windows/rustic-alpha-2_64-bit-popcnt.exe

64-bit bmi2 (Haswell CPU or newer):

rm -rf ./target && \
export RUSTFLAGS="-C target-cpu=haswell" && \
cargo build --release && \
strip -s ./target/release/rustic.exe && \
mv ./target/release/rustic.exe ./bin/windows/rustic-alpha-2_64-bit-bmi2.exe

64-bit native (Compiles for your current CPU):

rm -rf ./target && \
export RUSTFLAGS="-C target-cpu=native" && \
cargo build --release && \
strip -s ./target/release/rustic.exe && \
mv ./target/release/rustic.exe ./bin/windows/rustic-alpha-2_64-bit-native.exe
```

- You will find the binary in the ./bin/linux/ or ./bin/windows/ folder you
  just created.

# Extra module

There is a module called "Extra", which copmiles two extra capabilities into
the Rustic executable.

- Command-line option -e: Rustic can run a perft suite containing 172
  tests, to see if its move generator, make, and unmake are working as
  intended. This is mainly useful for developers.
- Command-line option -w: Using this option, Rustic can perform Wizardry:
  it runs a function that generates magic numbers for use in a magic
  bitboard engine, that have square A1 = 0, or LSB, and square H8 = 63, or
  MSB. This is mainly useful if one wants to write their own chess engine,
  bus has no interest in writing a function to compute the magic numbers.
  (Though, doing so, will make understanding of magic bitboards much more
  complete.)

This module can be included by using the --features option of cargo:

```
cargo build --release --features "extra"
```

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

There is one credit that doesn't have to do a lot with chess programming,
but it can't be forgotten. Now I finally understand why book writers always have
dedications to their spouses/children/families in their books.

* My girlfriend. Even though she is not a programmer, nor particularly
  interested in computers, I could not have written this chess engine
  without her. Just the fact that she's around when times are not the best,
  would be enough in itself, but she takes care of so much stuff in and
  around the house that without her, I wouldn't have even had the time to
  consider a project like this. In addition, she also helped in the
  development of the engine by listening to me blabbering about it, and
  then saying thigs like: "Maybe I'm asking a stupid question,
  but...",which helped me to avoid some stupid mistake more than once.

Many people on the [Talkchess.com](http://talkchess.com/forum3/index.php)
forum have provided insights and assistance that greatly helped in the
development of this chess engine. Using their experience and information
that has collected there over the years (and was posted as replies to my
questions), I was able to either avoid many bugs, wrong implementations, or
to improve existing code to work better, faster or cleaner. Thanks to all
of these people. Below is a list of people who I particularly remember for
one or more contributions (in no particular order).

- Richard Allbert (better known as BlueFever Software, author of VICE):
  his "Programming a Chess Engine in C" (or the Javascript version) series
  are legendary in the chess programming community. Richard may have
  instructed an entirely new generation of chess programmers.
- Terje Kirstihagen (author of Weiss): for the friendly Perft speed
  competition between an early version of Rustic and Weiss, to optimize
  make(), unmake(), and the move generator speed. And for his encouragement
  to keep going, by posting in my "Progress on Rustic" topic.
- Fabian von der Warth (author of FabChess): for giving helpful information
  about the Rust programming language, which were very useful in speeding
  up the early versions of the engine.
- H.G. Müller (author of FairyMax, MicroMax, and others): especially for
   explaining some improvements with regard to TT usage, and general
   assistance by patiently answering questions he must have seen countless
   times over the years.
- Maksim Korzh (author of Wukong and BBC): for writing a great video series
  in the same vein as the one written by Richard Allbert. While the series
  covers known ground, the perspective can be just from a different enough
  angle to make something 'click'.
- Rasmus Althoff (author of CT800): for assisting in optimizing Rustic's
  position repetition detection, and getting rid of some unneeded stuff in
   the alpha-beta and QSearch functions (and for providing many tidbits of
   useful information).
- Martin Sedlák (author of Cheng), and Eric Madsen (MadChess): Thanks for
   the pointers that put me on the right track to find out why TT Move
   sorting wasn't performing as expected.
- Ronald de Man (author of the Syzygy tablebases, CFish and RustFish): for
  helping to untangle the mess within my head regarding Principal Variation
  Search.
- Taimo (author of Monchester): for pointing out a potential variable underflow
  problem within the time management, that made Rustic crash in debug-mode.
- Sven Schüle (author of Jumbo, KnockOut, Surprise) for pointing out some lines of
  redundant, and thus confusing code in Rustic's search and qsearch functions.
- Ed Schröder (author of Rebel and Gideon) and Robert Hyatt (author of Cray
  Blitz and Crafty): for still hanging around chess forums, answering
  questions, even after writing chess engines for 40 or 50 years.

I always wanted to be "one of them": the programmers who could write chess
engines. I have always wanted to have my own, but there was always
"something" that got in the way of actually writing it. Now it's done: even
though it is not very strong yet, I wrote my own chess engine from scratch.
It was a long process and lots of work, but thanks to the help of the chess
programming community, it went relatively smoothly. Thanks guys.

