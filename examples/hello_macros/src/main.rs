#![feature(proc_macro)]

extern crate shio;

use shio::prelude::*;

// Simple requests should be simple, even in the face of asynchronous design.
#[get("/")]
fn hello_world(_: &Context) -> Response {
    Response::with("Hello World!\n")
}

#[get("/hello/{name}")]
fn hello(ctx: &Context) -> Response {
    Response::with(format!("Hello, {}!", &ctx.get::<Parameters>()["name"]))
}

#[get("/bye/{name}")]
fn bye(name: &String) -> Response {
    Response::with(format!("Bye, {}!", name))
}

#[get("/converse/{say}")]
fn converse(ctx: &Context, say: &String) -> Response {
    Response::with(format!("Bye, {}!", say))
}



fn main() {
    Shio::default()
        .route(hello_world)
        .route(hello)
        .route(bye)
        .run(":7878")
        .unwrap();
}
