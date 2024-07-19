"""
This module provides a parser to parse url and get the necessary information.
"""

import urllib
import urllib.parse

from rsdbpy.const import DEFAULT_HOST, DEFAULT_PORT
from rsdbpy.errors import InvalidUrlError


class URLParser:
    """a parser to parse url and get the necessary information"""

    VALID_SCHEMES = (
        "rsdb",
    )

    def __init__(self, link_url: str):
        self.url = link_url
        self.db_name = None
        self.parse_url()

    def parse_url(self):
        """解析url"""
        rs = urllib.parse.urlparse(self.url)
        self.scheme = rs.scheme
        if self.scheme not in self.VALID_SCHEMES:
            raise InvalidUrlError(self.url)
        self.username = rs.username
        self.password = rs.password
        self.hostname = rs.hostname or DEFAULT_HOST
        self.port = rs.port or DEFAULT_PORT
        if not self.hostname:
            raise InvalidUrlError(self.url, "hostname is required")
        if not isinstance(self.port, int) or self.port < 1 or self.port > 65535:
            raise InvalidUrlError(self.url, "invalid port")
        # self.path = rs.path
        self.get_db_name(rs.path)
        # self.query = rs.query
        # self.fragment = rs.fragment
        # self.params = rs.params

    def get_db_name(self, path_rs):
        """get database name"""
        if path_rs and path_rs.startswith("/"):
            db = path_rs[1:].strip()
            if len(db) > 0 and '/' not in db:
                self.db_name = db


if __name__ == '__main__':

    URL = "rsdb://user:pass@localhost:10000/testdb?charset=utf8&ssl=false#table1"
    parser = URLParser(URL)
    assert parser.scheme == "rsdb"
    assert parser.username == "user"
    assert parser.password == "pass"
    assert parser.hostname == "localhost"
    assert parser.port == 10000
    # assert parser.path == "/testdb"
    assert parser.db_name == "testdb"
    # assert parser.query == "charset=utf8&ssl=false"
    # assert parser.fragment == "table1"
    # assert parser.params == ""
