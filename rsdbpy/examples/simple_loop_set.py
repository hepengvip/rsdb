from rsdbpy import const, RsDbClient

client = RsDbClient(url="rsdb://@localhost/gdb")
for i in range(10000):
    key = "key" + str(i)
    value = "value" + str(i)
    rs = client.set(key, value)
    assert rs.resp_type == const.RESP_OK
    assert rs.msg == 'Ok.'
    print("set key: {}, value: {}".format(key, value))
client.close()
