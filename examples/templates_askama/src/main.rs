extern crate hyper;
extern crate shio;

#[macro_use]
extern crate askama;

use shio::prelude::*;
use askama::Template;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

// TODO: From what I understand, this should be put inside of Askama so it'll be autogenerated
//       Askama already does this for Rocket and Iron. Send a PR later.
impl<'a> shio::response::Responder for HelloTemplate<'a> {
    // FIXME: `type Error = _` shouldn't need to be declared or it should accept more
    //        than `hyper::Error`..
    type Error = hyper::Error;
    type Result = Response;

    fn to_response(self) -> Self::Result {
        Response::build().body(self.render())
    }
}

fn hello<'a>(_: Context) -> HelloTemplate<'a> {
    HelloTemplate { name: "George" }
}

fn main() {
    Shio::default()
        .route((Method::Get, "/hello", hello))
        .run(":7878")
        .unwrap()
}
