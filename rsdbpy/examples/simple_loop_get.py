from rsdbpy import const, RsDbClient

client = RsDbClient(url="rsdb://@localhost/gdb")
for i in range(10000):
    key = "key" + str(i)
    rs = client.get(key)
    assert rs.resp_type == const.RESP_TOKENS
    print(key, rs.tokens)
client.close()
