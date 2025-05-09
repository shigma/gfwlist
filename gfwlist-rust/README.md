# gfwlist

A fast GFW list parser and matcher.

## Installation

```bash
cargo add gfwlist
```

## Usage

```rust
use gfwlist::GfwList;

let list_content = "\
    ||blocked-site.com\n\
    @@||exception.com\n\
    /regex-pattern/\n\
";
let gfw = GfwList::from(list_content).unwrap();

assert_eq!(gfw.test("http://blocked-site.com/page").unwrap(), Some("||blocked-site.com"));
assert_eq!(gfw.test("http://exception.com/page").unwrap(), None);
assert_eq!(gfw.test("http://allowed-site.com/page").unwrap(), None);
```
