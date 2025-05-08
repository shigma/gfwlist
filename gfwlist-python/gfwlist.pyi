class GfwListSyntaxError(ValueError):
    pass

class GfwListBuildError(RuntimeError):
    pass

class GfwListUrlError(ValueError):
    pass

class GfwList:
    def __init__(self, rules_text: str) -> None: ...
    def test(self, url: str) -> bool: ...
    def __len__(self) -> int: ...
