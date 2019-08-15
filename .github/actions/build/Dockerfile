FROM rust:1.36.0

LABEL "com.github.actions.name"="Build"
LABEL "com.github.actions.description"="Build"
LABEL "com.github.actions.icon"="mic"
LABEL "com.github.actions.color"="purple"

RUN apt-get update && apt-get install -y sqlite3 libsqlite3-dev

ADD entrypoint.sh /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
