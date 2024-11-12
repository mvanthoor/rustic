
<!-- @import "[TOC]" {cmd="toc" depthFrom=1 depthTo=6 orderedList=false} -->

<!-- code_chunk_output -->

- [Building Rustic](#building-rustic)
  - [Installing the environment](#installing-the-environment)
    - [Windows](#windows)
    - [Linux (and probably most other Unix-like systems)](#linux-and-probably-most-other-unix-like-systems)
    - [MacOS](#macos)
  - [Compiling and building](#compiling-and-building)
  - [If "make" doesn't work](#if-make-doesnt-work)
  - [On 32-bit versions](#on-32-bit-versions)
- [Alternatives](#alternatives)

<!-- /code_chunk_output -->


# Building Rustic

As Rustic is open source, anyone can build their own executable or fork the
engine at any point in the development tree, to use it as a basis for their
own chess engine. To be able to create the executable, you need a proper
build environment.

Even though Rustic was developed on Windows, the GNU target is the default
instead of MSVC. As a shell, MSYS2's BASH version was used with all the
default Unix command-line tools available. Therefore it should be possible
to build Rustic in any Unix-like environment for which Rust is available,
and under MSYS2 in Windows.

## Installing the environment

Under most Unix-like operating systems such as Linux, the build environment
is already set up by default, especially if a Bash-compatible shell is
being used. To build software in Unix-style, Windows needs MSYS2 installed.

### Windows

First, download the Rust installer for Windows:

https://www.rust-lang.org/tools/install

Run through the installation. Make sure you select the GNU target as the
default. This is the target used to develop and test Rustic.

Second, download and install MSYS2. Installing GCC and Clang is not
_strictly_ necessary, but if you are compiling Rustic yourself there's some
chance you may want to build other chess engines as well. As most engines
are written in C or C++, it's very useful to have GCC and Clang installed
for compiling those engines.

```
- Download the installer: https://www.msys2.org/#installation
- Run the setup.
- Exit any console that may pop up after install.
- Start the MSYS2 - MSYS terminal (for maintenance)
- Right-click the title bar and select Options...
- Set up the terminal preferences how you want them.
- Run "pacman -Syu <enter>"
- Close the terminal when asked.
- Keep repeating the previous two steps until pacman says:
    - "Nothing more to do".
- Start MSYS2 - MinGW64 (for 64-bit compiles)
- Run these commands to install GCC and Clang:
    - pacman -S mingw64/mingw-w64-x86_64-gcc
    - pacman - S mingw64/mingw-w64-x86_64-clang
- Browse to the folder where you installed MSYS2.
    - In my case: "C:\Programs\MSYS2"
- Open "msys2_shell.cmd" in your favorite text editor. 
- Uncomment the line: "set MSYS2_PATH_TYPE=inherit" and save.
- Close and restart any MSYS2 shell that may have been open.
```

If you want to build a 32-bit version fo Rustic for some reason, you can
follow the above steps, but for the "mingw32" shell instead of "mingw64".
You'll obviously need to install the "mingw32", and 32-bit versions of the
above packages, in the mingw32 MSYS2 shell.

### Linux (and probably most other Unix-like systems)

Linux does not need a lot of setup. It is recommended you install the
latest Rust-version from their website, as the ones in the distribution's
repository can sometimes be several versions behind. You can find the
installation instructions here:

https://www.rust-lang.org/tools/install

In Linux the rest of the build requirements are already installed by
default. You can test this using the following commands:

```
$ bash --version
$ strip --version
```

Install these if they aren't already. As there are so many Linux
distributions, it is out of the scope of this page to describe the
installation for these tools.

### MacOS

Just like Linux, MacOS doesn't need much setup because it is a Unix-like
operating system. First, [Install Rust](https://www.rust-lang.org/) for
MacOS.

## Compiling and building

If the build environment has been installed correctly, it should be easy to
compile and build the engine. There are two ways of getting the source code
onto your system: git and download/unzip.

It is beyond the scope of this page to fully describe how Git has to be
installed and configured. There are many tutorials around the internet
providing this information. If you are using Git, you can clone the engine
in a directory on your system:

```
https://github.com/mvanthoor/rustic.git
```

If you don't use Git, you can also download the code and unzip the archive.
Go to [Rustic's GitHub Page](https://github.com/mvanthoor/rustic) and
switch to the branch or tag you want to compile. Then download the zip-file.

After you have the engine in a directory on your system, switch to that
directory in a terminal. (Go to the root directory, not inside the /src
directory.) Now you should be able to type:

```bash
cargo build --release
```

If all goes well, the engine will be built. You can switch into /target/release
directory, find the exectuable, and run:

```bash
strip -s ./rustic
```

to strip the debug symbols from the release version.

For MacOS it might be (depending on the version of strip):

```bash
strip ./target/release/rustic
```

## On 32-bit versions

Because a normal western chess board has 64 squares, the engine uses 64-bit
integers for most of its calculations, especially for move generation. Even
though Rustic can be built and run as a 32-bit engine, it will lose about
half of its speed, and will be 50-75 Elo weaker. It is not recommended to
use such a version for tournaments and matches. The only operating system
where the 32-bit version is the default is Raspberry Pi OS, because this
operating system is 32-bit. The 64-bit version of Raspberry Pi OS is still
experimental at the time of writing.

# Alternatives

On Windows, you can use the Rust MSVC target, but if you do, you will also
need to install the [C++ Build
Tools](https://visualstudio.microsoft.com/downloads/), because this target
needs the Microsoft Linker that comes with these tools. (Warning: the C++
build tools have a massive installation size.)

It is probably also possible to build Rustic on WSL2 (Windows Subsystem for
Linux 2) and on Cygwin, by installing the respective versions of Rust and
GNU tools there before either running "cargo build --release".

If built on Cygwin, the Rustic executable will depend on Cygwin1.dll. As
this is an additional layer between the engine and operating system, this
will lose some speed and thus playing strength.

I've not yet tried these ways of building the engine, so I have no data
with regard to the performance or stability of these binaries.  Cygwin will
probably never be tried because of the dependency on Cygwin1.dll