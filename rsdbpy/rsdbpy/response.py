"""
This module defines the Response class.
"""

from rsdbpy import const


class Response:
    """a response object from server"""
    def __init__(self, resp_type, *args, **kwargs):
        self.resp_type = resp_type
        assert self.validate_resp_type()
        self.args = args
        for k, v in kwargs.items():
            self.set_value(k, v)

    def set_value(self, k, v):
        """set a value of a key"""
        setattr(self, k, v)

    def validate_resp_type(self):
        """validate the response type"""
        return self.resp_type in const.RESP_TYPES
