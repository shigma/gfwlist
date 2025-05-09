# gfwlist

A fase GFW list parser and matcher.

## Installation

```bash
pip install gfwlist
```

## Usage

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
