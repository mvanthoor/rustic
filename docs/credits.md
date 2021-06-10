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
  redundant, and thus confusing code in Rustic's search and qsearch
  functions.
- Thomas (Lithander, author of MinimalChess): for the engaging discussions
  regarding (chess) programming, and providing another stable engine with
  compiles especially for me to test against.
- Ed Schröder (author of Rebel and Gideon) and Robert Hyatt (author of Cray
  Blitz and Crafty): for still hanging around chess forums, answering
  questions, even after writing chess engines for 40 or 50 years.

I always wanted to be "one of them": the programmers who could write chess
engines. I have always wanted to have my own, but there was always
"something" that got in the way of actually writing it. Now it's done: even
though it is not very strong yet, I wrote my own chess engine from scratch.
It was a long process and lots of work, but thanks to the help of the chess
programming community, it went relatively smoothly. Thanks guys.