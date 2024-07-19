import sys

from rsdbpy import const, RsDbClient

key = "key1"
if len(sys.argv) > 1:
    key = sys.argv[1]

client = RsDbClient(url="rsdb://@localhost/gdb")
rs = client.get(key)
assert rs.resp_type == const.RESP_TOKENS
assert rs.tokens == [b'value1']
print("al good here")
client.close()
