[![travis](https://travis-ci.org/daph/lada.svg?branch=master)](https://travis-ci.org/daph/lada/)

# lada
A slack markov bot written in rust

It reads it's initial corpus from corpus.txt in its working directory. Just put some fun sentences in there to train it on and start it up. It'll add new sentences it gets from slack messages sent to to its 'brain', so it's always learning!

Build with cargo, then run it:
```
$ git clone https://github.com/daph/lada.git
$ cd lada
$ cargo build --release
$ echo "this is my seed corpus" > seed.txt
$ ./target/release/lada --token <SLACK TOKEN>
```
