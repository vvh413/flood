# flood

Simple netowrk flood tool

```sh
$ flood -h
Network flood tool

Usage: flood [OPTIONS] <COMMAND>

Commands:
  icmp  ICMP (ping) flood
  udp   UDP flood
  syn   SYN flood
  help  Print this message or the help of the given subcommand(s)

Options:
  -t, --threads <THREADS>  Number of threads [default: 3]
  -d, --delay <DELAY>      Delay between packets in microseconds [default: 0]
  -h, --help               Print help
  -V, --version            Print version
```

## Installation

```sh
git clone https://github.com/vvh413/flood.git
cd flood
cargo install --path .
```
