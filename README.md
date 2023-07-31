# Rust Maelstrom

A solution for (https://fly.io/dist-sys/)[Fly.io Gossip Glomers] challenges

## Setup

1. Clone this repo
2. Install maelstrom from https://github.com/jepsen-io/maelstrom
3. Run the commands from below to test each solution
4. Run `./maelstrom/maelstrom serve` to inspect the output of each solution

## Echo

`./maelstrom/maelstrom test -w echo --bin target/debug/basic_node --node-count 1 --time-limit 10`

## Unique ID Generation

`./maelstrom/maelstrom test -w unique-ids --bin target/debug/basic_node --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition`

## Broadcast

### Challenge #3a: Single-Node Broadcast

`./maelstrom/maelstrom test -w broadcast --bin target/debug/broadcast_node --node-count 1 --time-limit 20 --rate 10`

### Challenge #3b: Multi-Node Broadcast

`./maelstrom/maelstrom test -w broadcast --bin target/debug/broadcast_node --node-count 5 --time-limit 20 --rate 10`

### Challenge #3c: Fault Tolerant Broadcast

`./maelstrom/maelstrom test -w broadcast --bin target/debug/broadcast_node --node-count 5 --time-limit 20 --rate 10 --nemesis partition`

### Challenge #3d: Efficient Broadcast, Part I

`./maelstrom/maelstrom test -w broadcast --bin target/debug/broadcast_node --node-count 25 --time-limit 20 --rate 100 --latency 100`
