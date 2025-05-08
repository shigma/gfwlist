class GfwListSyntaxError(ValueError):
    pass

class GfwListBuildError(RuntimeError):
    pass

class GfwListUrlError(ValueError):
    pass

class GfwList:
    def __init__(self, rules_text: str) -> None:
        """
        Create a new GfwList instance from rules text
        
        Args:
            rules_text (str): The text content of the GFW list rules
        
        Returns:
            GfwList: A new GfwList instance
        
        Raises:
            GfwListSyntaxError: If there's a syntax error in the rules
            GfwListBuildError: If there's an error building the pattern matching engine
        """
        ...
    def test(self, url: str) -> str | None:
        """
        Test if a URL matches any rule in the GfwList

        Args:
            url (str): The URL to test

        Returns:
            Optional[str]: The matching rule if found, otherwise None

        Raises:
            GfwListUrlError: If the URL is invalid or cannot be parsed
        """
        ...
    def __len__(self) -> int:
        """
        Get the number of rules in the GfwList

        Returns:
            int: The number of rules
        """
        ...
