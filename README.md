# Five Words

This project is inspired by Matt Parker's video ["Can you find: five five-letter
words with twenty-five unique letters?"](https://www.youtube.com/watch?v=_-AfhLQfb6w).

The idea is to find combinations of five-letter words such that they have no
letters in common. Anagrams of each of the words are not considered.

Matt [wrote a program](https://github.com/standupmaths/fiveletterworda) that
could do it in about 32 days. Benjamin Paassen [improved it](https://gitlab.com/bpaassen/five_clique)
to about 22 minutes.

This appeared to the author as optimizable.

## I gave it a go

I coded up a naive version in Rust, which instantly knocked the time down to
about 3 minutes on my machine.

After this I spent some time using the excellent Rayon crate to parallelize it,
further shaving it down to about 44 seconds on my machine. Just to see what
would happen, I tried it on a 64-core AWS EC2 instance, on which it ran in
about 3.7 seconds. By my calculations, that's 352 times as fast as Paassen's
time of 21.75 minutes. But it's giving it a go that counts!

## How to run it

This repo uses the Rust 2021 edition, which is not in all package repos yet. The
easiest way to compile and run it for yourself is probably to install Rust
using the [Rustup](https://rustup.rs/) toolchain installer, navigating to the
project root, and then running:

    cargo run --release -- -p /path/to/words_alpha.txt
