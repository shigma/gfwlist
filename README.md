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
let gfw_list = GfwList::from(list_content).unwrap();

assert_eq!(gfw_list.test("http://blocked-site.com/page").unwrap(), Some("||blocked-site.com"));
assert_eq!(gfw_list.test("http://exception.com/page").unwrap(), None);
assert_eq!(gfw_list.test("http://allowed-site.com/page").unwrap(), None);
```

## Python

```python
from gfwlist import GFWList

list_content = """
||blocked-site.com
@@||exception.com
/regex-pattern/
"""
gfwlist = GFWList(list_content)

assert gfwlist.test("http://blocked-site.com/page") == "||blocked-site.com"
assert gfwlist.test("http://exception.com/page") is None
assert gfwlist.test("http://allowed-site.com/page") is None
```
