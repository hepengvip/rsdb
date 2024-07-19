"""
This module contains all the custom errors used in rsdbpy.
"""

class BaseError(Exception):
    """base class for all custom errors"""


class InvalidUrlError(BaseError):
    """raised when an invalid URL is passed to a function"""
    def __init__(self, url, reason=None):
        self.url = url
        super().__init__(f"Invalid URL: {url}  {reason}")


class ConnectionClosedError(BaseError):
    """raised when a connection is closed"""
    def __init__(self):
        super().__init__("Connection closed")


class UnknownResponse(BaseError):
    """raised when an unknown response is received"""
    def __init__(self, resp):
        super().__init__(f"Unknown response: {resp}")


class OpError(BaseError):
    """raised when an operation fails"""
    def __init__(self, msg):
        super().__init__(f"Operation failed: {msg}")
