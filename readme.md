# Ping Graph

This is my first attempt doing some coding in Rust. I'm still experimenting with
different language features and constructs.

The goal is to continuously ping an IP address, creating a report to plot some
nice graphs. The data concerns latency as well as packet loss.

The tool creates two files, one with the full set of data, the other aggregates
the data in 10min buckets.