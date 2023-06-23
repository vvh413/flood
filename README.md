# flood

Simple netowrk flood tool

```sh
$ flood -h
Network flood tool

Usage: flood [OPTIONS] <HOST>

Arguments:
  <HOST>  Destination address

Options:
  -s, --size <SIZE>        Packet size in bytes [default: 1471]
  -t, --threads <THREADS>  Number of threads [default: 3]
  -d, --delay <DELAY>      Delay between packets in microseconds [default: 0]
  -h, --help               Print help
  -V, --version            Print version
```

## Installation

```sh
$ git clone https://github.com/vvh413/flood.git
$ cargo install --path .
```
