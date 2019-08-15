#[macro_use]
extern crate clap;

use actix::System;
use actix_web::{error, middleware, web, App, HttpResponse, HttpServer};
use crypto::{digest::Digest, sha2::Sha256};
use csv::ReaderBuilder;
use log::LevelFilter;
use log::{debug, error, warn};
use r2d2;
use r2d2_sqlite;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection};
use serde;
use serde::{Deserialize, Serialize};
use std::io;

type Range = (i64, i64, i64, i64);

fn to_hash(i: i64) -> (i64, String) {
    let mut hasher = Sha256::new();
    hasher.input_str(&i.to_string());
    (i, hasher.result_str())
}

fn create_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS data (
            hash VARCHAR(128) NOT NULL,
            value UNSIGNED BIG NOT NULL
        );",
        params![],
    )?;

    Ok(())
}

fn create_index(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS hash_idx ON data (hash);",
        params![],
    )?;

    Ok(())
}

fn drop_index(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("DROP INDEX IF EXISTS hash_idx;", params![])?;

    Ok(())
}

fn insert_range(conn: &mut Connection, r: Range) -> Result<(), rusqlite::Error> {
    let tx = conn.transaction()?;

    (r.0..=r.1)
        .map(|x| r.2 * 10_i64.pow(1 + (x as f64).log10().floor() as u32) + x)
        .map(|x| r.3 * 10_i64.pow(1 + (x as f64).log10().floor() as u32) + x)
        .map(to_hash)
        .try_for_each(|x| -> Result<(), rusqlite::Error> {
            tx.execute(
                "INSERT INTO data (value, hash) VALUES (?1, ?2)",
                params![x.0, x.1],
            )?;
            Ok(())
        })?;

    tx.commit()
}

fn make_ranges_form_file(
    path: &str,
    sep: u8,
    global_prefix: i64,
) -> Result<(Vec<Range>), Box<std::error::Error>> {
    let mut ranges = vec![];

    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(sep)
        .flexible(true)
        .from_path(path)
        .or_else(|e| match e.kind() {
            csv::ErrorKind::Io(ref err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    let msg = format!("No such file {}", &path);
                    return Err(csv::Error::from(io::Error::new(
                        io::ErrorKind::NotFound,
                        msg,
                    )));
                }
                return Err(e);
            }
            _ => return Err(e),
        })?;

    for result in rdr.records() {
        let record = result?;
        if record.len() < 2 {
            warn!("Length line {:?} must have min 2 fields. Skip", record);
            continue;
        }

        if record.len() == 3 {
            ranges.push((
                record[1].parse::<i64>()?,
                record[2].parse::<i64>()?,
                record[0].parse::<i64>()?,
                global_prefix,
            ));
        } else if record.len() == 2 {
            ranges.push((
                record[0].parse::<i64>()?,
                record[1].parse::<i64>()?,
                0,
                global_prefix,
            ));
        }
    }

    Ok(ranges)
}

fn generate(matches: &clap::ArgMatches) -> Result<(), Box<std::error::Error>> {
    let mut ranges = vec![];

    if matches.is_present("paths") {
        let in_sep: &str = matches.value_of("sep").unwrap_or(",");
        let sep = sep_as_byte(in_sep)?;
        let paths: Vec<_> = matches.values_of("paths").ok_or("Bad paths")?.collect();
        let prefix = value_t!(matches, "global-prefix", i64)?;

        for path in paths {
            ranges.extend(make_ranges_form_file(path, sep, prefix)?);
        }
    }

    if matches.is_present("start") {
        ranges.push((
            value_t!(matches, "start", i64)?,
            value_t!(matches, "end", i64)?,
            value_t!(matches, "prefix", i64)?,
            value_t!(matches, "global-prefix", i64)?,
        ));
    }

    let path = matches.value_of("sqlite").unwrap();
    let mut conn = Connection::open(&path)?;

    debug!("Create table");
    create_table(&conn)?;
    drop_index(&conn)?;

    ranges
        .into_iter()
        .map(|r| {
            debug!(
                "Generate range {}-{} with prefix {} / {}",
                r.0, r.1, r.3, r.2
            );
            r
        })
        .try_for_each(|r| insert_range(&mut conn, r))?;

    debug!("Create index");
    create_index(&conn)?;

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct Hash {
    hash: String,
    value: i64,
}

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

fn index(
    input_hash: web::Json<Vec<String>>,
    pool: web::Data<Pool>,
) -> actix_web::Result<HttpResponse> {
    let conn = pool.get().unwrap();
    let mut hashs = vec![];

    for hash in input_hash.iter() {
        if let Err(err) = conn.query_row(
            "SELECT hash, value FROM data WHERE hash = ?1;",
            params![hash],
            |row| {
                let h = Hash {
                    hash: row.get(0)?,
                    value: row.get(1)?,
                };
                hashs.push(h);
                Ok(())
            },
        ) {
            match err {
                rusqlite::Error::QueryReturnedNoRows => {}
                _ => return Err(error::ErrorInternalServerError(err)),
            }
        };
    }

    Ok(HttpResponse::Ok().json(hashs))
}

fn sep_as_byte(rec_str: &str) -> Result<u8, clap::Error> {
    let bytes = rec_str.as_bytes();
    if bytes.len() == 1 {
        Ok(bytes[0])
    } else {
        let e = clap::Error {
            message: "Input separator must be encodable to 1 byte exactly!".to_owned(),
            kind: clap::ErrorKind::ValueValidation,
            info: None,
        };
        Err(e)
    }
}

fn server(matches: &clap::ArgMatches) -> Result<(), Box<std::error::Error>> {
    let path = matches.value_of("sqlite").unwrap();
    let http_addr = matches.value_of("http-addr").unwrap();
    let manager = SqliteConnectionManager::file(path);
    let pool = Pool::new(manager)?;

    let sys = System::new("rainbow");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(pool.clone())
            .route("/", web::post().to(index))
    })
    .bind(http_addr)
    .unwrap()
    .start();

    let _ = sys.run();

    Ok(())
}

fn main() {
    let env = env_logger::Env::new().filter_or("RAINBOW_LOG", "info");
    env_logger::Builder::from_env(env)
        .default_format()
        .filter_module("actix_web", LevelFilter::Info)
        .default_format_module_path(false)
        .init();

    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml).get_matches();

    match matches.subcommand() {
        ("generate", Some(args)) => {
            generate(args).unwrap_or_else(|err| error!("{}", err));
        }
        ("server", Some(args)) => {
            server(args).unwrap_or_else(|err| error!("{}", err));
        }
        ("", None) => error!("No subcommand was used"),
        _ => unreachable!(),
    }
}
