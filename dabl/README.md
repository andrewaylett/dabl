dabl
========

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/andrewaylett/dabl/Rust)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-v2.0%20adopted-ff69b4.svg)](../code_of_conduct.md)
[![dependency status](https://deps.rs/repo/github/andrewaylett/dabl/status.svg)](https://deps.rs/repo/github/andrewaylett/dabl)

Looks up IP addresses and domain names in so-called "DNSRBLs".
I say "so-called" because there's no real reason why they should be _block_ lists.

This project takes significant inspiration (but no code) from https://github.com/logic/rblcheck.
The biggest benefit over the original is IPv6 support, which is unfortunately lacking from most RBL tooling.
We also support allow-lists, and if an IP or name is found in one of the allow-lists then we report not blocked.
There's no short-circuiting, we'll check all the lists for both IPs and Names, and also check all the block-lists even if we found an entry in an allow-list.

There are currently no lists in the "standard" set, and the SpamHaus key doesn't actually do anything: watch this space.

Usage
-----

```
$ dabl --help
dabl 0.1.0
Andrew Aylett <andrew@aylett.co.uk>
Queries DNS Allow- or Block-Lists

USAGE:
    dabl [FLAGS] [OPTIONS] [--] [query]...

FLAGS:
        --standard    Use the built-in 'standard' set of lists
    -h, --help        Prints help information
    -V, --version     Prints version information

OPTIONS:
    -a, --allow <IP Allow>...            Specify an allow list to use
    -b, --block <IP Block>...            Specify a block list to use
        --spamhaus-key <SpamHaus key>    Your SpamHaus subscription key

ARGS:
    <query>...
```
