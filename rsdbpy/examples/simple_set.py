from rsdbpy import const, RsDbClient

client = RsDbClient(url="rsdb://@localhost/gdb")
rs = client.set("key1", "value1")
assert rs.resp_type == const.RESP_OK
assert rs.msg == 'Ok.'
client.close()
