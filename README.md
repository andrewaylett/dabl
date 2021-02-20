rblcheck
========

[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-v2.0%20adopted-ff69b4.svg)](code_of_conduct.md)

Looks up IP addresses and domain names in so-called "DNSRBLs".
I say "so-called" because there's no real reason why they should be _block_ lists.

This project takes significant inspiration (including its CLI interface, but no code) from https://github.com/logic/rblcheck.
The biggest benefit over the original is IPv6 support, which is unfortunately lacking from most RBL tooling.

Usage
-----

```
$ rblcheck --help
rblcheck 0.1.0
Andrew Aylett <andrew@aylett.co.uk>
Queries DNS block-lists (or allow lists!)

USAGE:
    rblcheck [FLAGS] [OPTIONS] [--] [query]...

FLAGS:
    -c               Clear the built-in list
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -s <source>...        Specify a list to use

ARGS:
    <query>...
```
