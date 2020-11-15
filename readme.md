![Rustic Banner](https://rustic-chess.org/img/rustic-logo-web.jpg)

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

At the time of writing (15-11-2020) Rustic does not yet have many features,
if any at all. The basiscs to be able to play a decent game of chess have
been implemented. This will be the base version for establishing an initial
Elo rating, before more features are added. This is done so the Elo-gain of
each added feature can be determined. The current feature set is:

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

There are many features that will be added in the future, such as:
- Finishing the XBoard protocol.
- Several pruning options in the search
- Add a transposition table
- option/setoption in UCI to control engine options
- Tapered evaluation for middle game/endgame
- Many more evaluation terms
- Add Lazy SMP

These will be listed under "Features" as they are implemented.

# Quick compiling tips

Follow the instructions below if you want to compile the engine yourself.

- First install [Rust](https://www.rust-lang.org/) for your platform. Make
  sure you install at least version 1.46, as Rustic uses featurse that only
  became available in that version.
- Make sure you install the correct toolchain for your platform.
- If you are running Windows, it is recommended to have
  [MSYS2](https://www.msys2.org/) installed, because it provides Bash. If
  you have Git and Git Bash installed, you can possibly omit MSYS2.
- Clone (or download/unzip) this repository onto your computer.
- Change to the root folder of the repository (where cargo.toml is).
- Run the following command, without quotes:
  - "cargo build --release"
- You will find the binary in "./target/release/", called "rustic(.exe)".
- You can strip any debug symbols from the binary using "strip":
  - "strip -s ./target/release/rustic" (for Linux/Unix/Mac)
  - "strip -s ./target/release/rustic.exe" (for Windows)
- The above will give you a generic executable (64-bit, assuming you are
  compilng to a 64-bit target, which is highly recommended). If you want to
  make the compile optimized for the CPU you're compiling on, set the
  following environment variable before compiling:
    - RUSTFLAGS='-C target-cpu=native'

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
development of this chess engine. Using their experience and information, I
was able to either avoid many bugs, wrong implementations, or to improve
existing code to work better, faster or cleaner. Thanks to all of these
people. Below is a list of people who I particularly remember for one or
more contributions (in no particular order).

* Richard Allbert (better known as BlueFever Software, author of VICE):
  his "Programming a Chess Engine in C" (or the Javascript version) series
  are legendary in the chess programming community. Richard may have
  instructed an entirely new generation of chess programmers.
* Terje Kirstihagen (author of Weiss): for the friendly Perft speed
  competition between an early version of Rustic and Weiss, to optimize
  make(), unmake(), and the move generator speed. And for his encouragement
  to keep going, by posting in my "Progress on Rustic" topic.
* Fabian von der Warth (author of FabChess): for giving helpful information
  about the Rust programming language, which were very useful in speeding
  up the early versions of the engine.
* H.G. Müller (author of FairyMax, MicroMax, and others): especially for
  pointing out some improvements with regard to hash table usage, and
  general assistance by patiently answering questions he must have seen
  countless times over the years.
* Maksim Korzh (author of Wukong and BBC): for writing a great video series
  in the same vein as the one written by Richard Allbert. While the series
  covers known ground, the perspective can be just from a different enough
  angle to make something 'click'.
* Rasmus Althoff (author of CT800): for assisting in optimizing Rustic's
  position repetition detection, and getting rid of some unneeded stuff in
  the alpha-beta and QSearch functions.
* Martin Sedlák (author of Cheng): I remember he answered a few questions I
  had, but I can't remeber which ones... sorry.
* Ronald de Man (author of the Syzygy tablebases, CFish and RustFish): for
  helping to untangle the mess within my head regarding Principal Variation
  Search.
* Taimo ("unserializable"): for pointing out a potential variable underflow
  problem within the time management, that made Rustic crash in debug-mode.
* Ed Schröder (author of Rebel and Gideon) and Robert Hyatt (author of Cray
  Blitz and Crafty): for still hanging around chess forums, answering
  questions, even after writing chess engines for 40 or 50 years.

I always wanted to be "one of them": the programmers who could write chess
programs. I have always wanted to have my own chess engine, but there was
always "something" that got in the way of actually writing it.

Now it's done: even though it is not very strong yet, I wrote my own chess
engine from scratch. It was a long process and lots of work, but thanks to
the help of the chess programming community, it went relatively smoothly.
Thanks guys.

