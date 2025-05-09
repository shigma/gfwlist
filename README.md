# gfwlist

A fast GFW list parser and matcher.

## Rust

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

## Python

```python
from gfwlist import GFWList

list_content = """
||blocked-site.com
@@||exception.com
/regex-pattern/
"""
gfw = GFWList(list_content)

assert gfw.test("http://blocked-site.com/page") == "||blocked-site.com"
assert gfw.test("http://exception.com/page") is None
assert gfw.test("http://allowed-site.com/page") is None
```
