### Miscellaneous Tools

![Continuous integration](https://github.com/ccsplit/misc_tools/workflows/Continuous%20integration/badge.svg) ![Security audit](https://github.com/ccsplit/misc_tools/workflows/Security%20audit/badge.svg)

Mainly these tools are here to learn the syntax and working with [rust](https://doc.rust-lang.org/cargo/index.html).
Therefore, over time more and more tools may end up getting here. Currently working on just some simple one-off scripts
and/or porting some of my other scripts to Rust if applicable.

#### check_urls
A tool to check a list of URLs and return the valid URLs to the console. If you specify the `-o|--output` flag it will also
place them within the specified file.

#### ip_hostname
A tool to take a list/file of IPs, CIDRs etc and obtain the hostname for each of them using the system's DNS servers. _Currently
it does not support using other resolvers. **I need to investigate how this can be done in Rust**_
