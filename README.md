# rainbow

Rainbow is a HTTP service and generator that makes using rainbow tables easy.

## Usage

#### Generate table from file

```console
foo@bar:~$ cat ranges.csv
222,10,12
222,20,22
222,30,32
```

```console
foo@bar:~$ rainbow generate --sqlite my.db --file-with-range ranges.csv --global-prefix 11
```

```console
foo@bar:~$ sqlite3 my.db
sqlite> SELECT * FROM data;
56457d65707182874defc58b4fd452ba727f0825b5c8a37ebf82d164c03e2aeb|1122210
8795adee6073128d91ed55e90277257cbc99f19e97a876bc13fcf8432220006f|1122211
1b682f1353ced9617b88699ee7aadf7b029e71fa09b34a66c2eab76043a3c957|1122212
99ee4bdbfd0894db2085476848abf5ae4840671947243d5c7383d54c99db2c6d|1122220
ef1fea811b7ad857502ed181c9a57536fa42d3c999b337fd9754cac5119fdacc|1122221
b8b2f487fe6ff4658df05265c9e0af58f8b7617cf17f19a4cb1be474058b3e87|1122222
4c7ae6f863448f910fd28fc0dc295da1e7d4e50b4cf273527f2995fd3c2e4111|1122230
3ec9723fe4c3275a65f9b95a7b92eb32dddd18941bfab56604cae14519f2aa48|1122231
03b50c5e8ed6b7c2608e94c06430e4daa5248fbc078d76bac8ac15e679e34ad2|1122232
sqlite> 
```

#### Generate from cli interface

```console
foo@bar:~$ rainbow generate --sqlite my.db --start 10 --end 12 --prefix 222 --global-prefix 11
```

```console
foo@bar:~$ sqlite3 my.db
sqlite> SELECT * FROM data;
56457d65707182874defc58b4fd452ba727f0825b5c8a37ebf82d164c03e2aeb|1122210
8795adee6073128d91ed55e90277257cbc99f19e97a876bc13fcf8432220006f|1122211
1b682f1353ced9617b88699ee7aadf7b029e71fa09b34a66c2eab76043a3c957|1122212
sqlite> 
```

#### Use HTTP service

```console
foo@bar:~$ rainbow server --sqlite my.db
[2019-08-15T06:56:43Z INFO ] Starting 4 workers
[2019-08-15T06:56:43Z INFO ] Starting server on 127.0.0.1:8088
```

```console
foo@bar:~$ curl -s -X POST -H 'Content-Type: application/json' 'http://127.0.0.1:8088/' -d'["8795adee6073128d91ed55e90277257cbc99f19e97a876bc13fcf8432220006f"]'
[{"hash":"8795adee6073128d91ed55e90277257cbc99f19e97a876bc13fcf8432220006f","value":1122211}]
```