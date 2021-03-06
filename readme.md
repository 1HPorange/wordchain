# wordchain

wordchain is a command-line tool (and library) that can find the longest chain of 
partially overlapping words in a given list of words. Why you would ever want that?
I don't know. But now you have it.

## What does that even mean

Let's say you have a text file with a bunch of names in each line:

```
Jacob
Michael
Joshua
Matthew
Christopher
Andrew
Daniel
Ethan
Joseph
William
Anthony
Nicholas
David
Alexander
Ryan
Tyler
James
John
Jonathan
Brandon
```

then you can execute wordchain and you will get:

```
Longest chain (6): JoshuAlexandeRyAndreWilliaMichael
```

Isn't that just great?

## Usage

The tool has pretty good command-line help text. Just try `wordchain -h`. 
Try it out for some small workloads before giving it a long list.

Note that because of reasons, the application won't accept lists longer 
than 256 words. But that would take an eternity to calculate anyway, trust me.

### Tips

- Tweaking the value of the `--granularity` flag is pretty important. 
Try finding a good value for small and intermediate workloads first and 
increase that value for larger workloads.
- The `--quick-results` switch really degrades the overall execution time, so 
use it only when you can live with a good guess for the longest chain.

## Building

You will need a more-or-less recent version of the rust toolchain, 
[available from rustup](https://rustup.rs/).

```shell
git clone https://github.com/1HPorange/wordchain.git
cd wordchain
cargo build --release
```

The executable can be found in `target/release/`.

For secret omega-turbo ultra-boost, consider compiling 
specifically for your native environment by setting the 
environment variable `RUSTFLAGS="-C target-cpu=native"`. Just
remember to not share the executable after that, or your PC
will explode.

## Optimizations

To run as fast as possible, wordchain uses some optimizations:

- never operates on strings directly, only byte-size vector indices
- is non-recursive to avoid stack-related performance issues
- is parallelized with a configurable granularity to avoid load-balancing issues
- builds lookup structures up-front to avoid recalculation
- sorts words heuristically to shorten expected runtime
- runs entirely lock-free
- uses bitmasks to emulate a hashset with perfect hashing, which is used to avoid cycles
- uses safe estimates of longest chains with a specific first word to potentially abort 
later words early

## Tasks

- [x] Validate CL arguments. Right now I trust you. I shouldn't.