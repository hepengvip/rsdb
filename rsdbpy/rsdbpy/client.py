"""
Client for rsdb kv store.
"""

import socket
import struct

from rsdbpy import const
from rsdbpy import errors
from rsdbpy.response import Response
from rsdbpy.urlparser import URLParser


class RsDbClient:
    """client for rsdb kv store."""

    def __init__(
            self,
            url=None,
            host=const.DEFAULT_HOST, port=const.DEFAULT_PORT,
            db_name=None,
            # user=None, password=None,
    ):
        if url:
            rs = URLParser(url)
            self.host = rs.hostname
            self.port = rs.port
            self.db_name = rs.db_name
        else:
            self.host = host
            self.port = port
            self.db_name = db_name

        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.connect((self.host, self.port))
        self.sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        if self.db_name:
            self.use(self.db_name)

    def use(self, db_name):
        """using a database"""
        self._write_header(const.CMD_USE)
        self._write_token(self._ensure_bytes(db_name), last=True)
        return self.read_response()

    def set(self, k, v):
        """set a key-value pair"""
        return self.mset({k: v})

    def mset(self, data_map):
        """set multiple key-value pairs"""
        assert isinstance(data_map, dict)
        assert len(data_map) > 0
        self._write_header(const.CMD_WRITE)
        self._write_size(len(data_map))
        for (idx, (k, v)) in enumerate(data_map.items()):
            k = self._ensure_bytes(k)
            v = self._ensure_bytes(v)
            self._write_token(k)
            if idx == len(data_map) - 1:
                self._write_token(v, last=True)
            else:
                self._write_token(v)
        return self.read_response()

    def get(self, *k_s):
        """get a value by key"""
        assert len(k_s) > 0
        self._write_header(const.CMD_READ)
        self._write_size(len(k_s))
        for(idx, k) in enumerate(k_s):
            k = self._ensure_bytes(k)
            if idx == len(k_s) - 1:
                self._write_token(k, last=True)
            else:
                self._write_token(k)
        return self.read_response()

    def delete(self, *k_s):
        """delete a key-value pair"""
        assert len(k_s) > 0
        self._write_header(const.CMD_DELETE)
        self._write_size(len(k_s))
        for (idx, k) in enumerate(k_s):
            k = self._ensure_bytes(k)
            if idx == len(k_s) - 1:
                self._write_token(k, last=True)
            else:
                self._write_token(k)
        return self.read_response()

    def current_db(self):
        """get current database name"""
        self._write_header(const.CMD_CURRENT_DB, last=True)
        return self.read_response()

    def list_db(self):
        """list all databases currently attached"""
        self._write_header(const.CMD_LIST_DB, last=True)
        return self.read_response()

    def detach(self, db_name):
        """detach a database"""
        self._write_header(const.CMD_DETACH)
        self._write_token(self._ensure_bytes(db_name), last=True)
        return self.read_response()

    def read_response(self) -> bool:
        """read response from server"""
        val = self._read_header()
        if val == const.RESP_OK:
            msg = self._read_token().decode()
            return Response(const.RESP_OK, msg=msg)
        if val == const.RESP_ERROR:
            msg = self._read_token().decode()
            raise errors.OpError(msg)
        if val == const.RESP_TOKEN:
            byt = self._read_token()
            return Response(const.RESP_TOKEN, token=byt)
        if val == const.RESP_TOKENS:
            length = self._read_size()
            tokens = [self._read_token() for i in range(length)]
            return Response(const.RESP_TOKENS, tokens=tokens)
        if val == const.RESP_PAIRS:
            length = self._read_size()
            pairs = [(self._read_token(), self._read_token()) for i in range(length)]
            return Response(const.RESP_PAIRS, pairs=pairs)
        raise errors.UnknownResponse(val)

    def _check_connection(self):
        """check if connection is closed"""
        if self.sock is None:
            raise errors.ConnectionClosedError()

    def _read_header(self):
        """read a packet header"""
        self._check_connection()
        byt = self.sock.recv(const.CMD_LENGTH)
        f, = struct.unpack(">B", byt)
        return f

    def _write_header(self, val, last=False):
        """write a packet header"""
        self._check_connection()
        byt = val.to_bytes(const.CMD_LENGTH, 'big')
        if last:
            self.sock.sendall(byt)
        else:
            self.sock.send(byt)

    def _read_size(self):
        """read a size value"""
        self._check_connection()
        s = self.sock.recv(const.LEN_LENGTH)
        x, = struct.unpack(">h", s)
        return x

    def _write_size(self, val, last=False):
        """write a size value"""
        self._check_connection()
        val = struct.pack('>h', val)
        if last:
            self.sock.sendall(val)
        else:
            self.sock.send(val)
        # assert rs == LEN_LENGTH

    def _read_token(self):
        """read a token from stream"""
        self._check_connection()
        l_byt = self.sock.recv(const.TOKEN_LENGTH)
        length, = struct.unpack(">I", l_byt)
        if length == 0:
            return None
        data = self.sock.recv(length)
        assert len(data) == length, "incompleted length"
        return data

    def _write_token(self, data, last=False):
        """write a token to stream"""
        self._check_connection()
        length = len(data)
        l_byt = struct.pack('>I', length)
        assert self.sock.send(l_byt) == const.TOKEN_LENGTH
        if last:
            self.sock.sendall(data)
        else:
            assert self.sock.send(data) == length

    @staticmethod
    def _ensure_bytes(data):
        """convert data to bytes if it is not already"""
        if isinstance(data, bytes):
            return data
        if isinstance(data, str):
            return data.encode()
        raise ValueError("unknown type")

    def close(self):
        """close the connection"""
        self.sock.close()
        self.sock = None
