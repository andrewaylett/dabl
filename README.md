# dabl

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/andrewaylett/dabl/.github/workflows/rust.yml?branch=main)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-v2.0%20adopted-ff69b4.svg)](code_of_conduct.md)
[![dependency status](https://deps.rs/repo/github/andrewaylett/dabl/status.svg)](https://deps.rs/repo/github/andrewaylett/dabl)

Looks up IP addresses and domain names in so-called "DNSRBLs".
I say "so-called" because there's no real reason why they should be _block_ lists.

This project takes significant inspiration (including the CLI interface for the `rblcheck` executable, but no code) from https://github.com/logic/rblcheck.
The biggest benefit over the original is IPv6 support, which is unfortunately lacking from most RBL tooling.
We also support allow-lists, and if an IP or name is found in one of the allow-lists then we report not blocked.
There's no short-circuiting, we'll check all the lists for both IPs and Names, and also check all the block-lists even if we found an entry in an allow-list.

There are currently no lists in the "standard" set, and the SpamHaus key doesn't actually do anything: watch this space.

## Usage

<!-- [[[cog
result = sp.run(
    ["cargo", "run", "--bin", "dabl", "--", "--help"],
    capture_output=True,
    text=True,
    check=True
)
cog.outl(f"""
```
$ dabl --help
{result.stdout.strip()}
```
""")
]]] -->

```
$ dabl --help
dabl 0.5.1

USAGE:
    dabl [FLAGS] [OPTIONS] <query>

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      Silence all output
    -V, --version    Prints version information
    -v, --verbose    Output more details

OPTIONS:
    -a, --allow <allow>...    A DNS allow list
    -b, --block <block>...    A DNS block list
    -t, --timestamp <ts>      Timestamp (sec, ms, ns, none)

ARGS:
    <query>    An IP address (v4 or v6) or domain name
```

<!-- [[[end]]] -->

<!-- [[[cog
result = sp.run(
    ["cargo", "run", "--bin", "rblcheck", "--", "--help"],
    capture_output=True,
    text=True,
    check=True
)
cog.outl(f"""
```
$ rblcheck --help
{result.stdout.strip()}
```
""")
]]] -->

```
$ rblcheck --help
rblcheck 0.5.1

USAGE:
    rblcheck [FLAGS] [OPTIONS] [--] [addresses]...

FLAGS:
    -c               Clear the current list of DNSBL services
    -h, --help       Prints help information
    -l               List default DNSBL services to check
    -m               Stop checking after first address match in any list
    -q               Quiet mode; print only listed addresses
    -t               Print a TXT record, if any
    -V, --version    Prints version information

OPTIONS:
    -s <services>...        Toggle a service to the DNSBL services list

ARGS:
    <addresses>...    An IP address to look up; specify `-' to read multiple addresses from standard input
```

<!-- [[[end]]] -->
