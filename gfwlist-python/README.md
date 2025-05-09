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
gfw = GFWList(list_content)

assert gfw.test("http://blocked-site.com/page") == "||blocked-site.com"
assert gfw.test("http://exception.com/page") is None
assert gfw.test("http://allowed-site.com/page") is None
```
