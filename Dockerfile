FROM japaric/x86_64-unknown-linux-gnu:latest

RUN apt-get update && apt-get install -y sqlite3 libsqlite3-dev