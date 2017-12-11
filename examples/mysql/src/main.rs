extern crate pretty_env_logger;
#[allow(unused_extern_crates)]
extern crate serde;
extern crate serde_json;
extern crate shio;

extern crate mysql_async as my;

extern crate uuid;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate error_chain;

extern crate http;
use shio::prelude::*;
use my::prelude::*;

mod errors {
    error_chain! {
        foreign_links {
            Mysql(::my::errors::Error);
            Json(::serde_json::Error);
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
struct User {
    host: String,
    user: String,
}

const DATABASE_URL: &'static str = "mysql://root:admin@127.0.0.1:3306/mysql";

fn users(ctx: Context) -> BoxFuture<Response, ::my::errors::Error> {

    let pool = my::Pool::new(DATABASE_URL, &ctx.handle());

    pool.get_conn().and_then(|conn| {
        conn.prep_exec("select Host, User from user", ())
    })
    .and_then(|result| {
        result.map(|row| {
            User {
                host: row.get("Host").unwrap(),
                user: row.get("User").unwrap(),
            }
        })
    })
    .and_then(|result: (my::QueryResult<my::Conn, my::BinaryProtocol>, std::vec::Vec<User>)| {
        let s = serde_json::to_string(&result.1).expect("didn't work");

        Ok(
            Response::build()
                .body(s),
        )
    })
    .into_box()
}

fn main() {
    pretty_env_logger::init().unwrap();

    Shio::default()
        .route((Method::GET, "/users", users))
        .run(":7878")
        .unwrap();
}