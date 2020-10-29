//! Loadtest-Dashboard delivered on the edge!
#[macro_use] extern crate anyhow;
use anyhow::{Error, Result};
use fastly::http::{HeaderValue, Method, StatusCode};
use fastly::request::CacheOverride;
use fastly::{Body, Request, RequestExt, Response, ResponseExt};
#[macro_use] extern crate horrorshow;
use horrorshow::prelude::*;
use horrorshow::helper::doctype;

mod burden;
use burden::BurdenRequest;


fn root_page() -> Result<String> {
    let loadtest_result = BurdenRequest::new("5-minute-jam", "1597307460")?.get_response()?;
    Ok(format!("{}", html! {
        : doctype::HTML;
        html {
            head {
                title : "Hello world!";
            }
            body {
                p {
                    // Insert raw text (unescaped)
                    : Raw(format!("Hello, <i>Colin</i>! This was the avg response time: {}", loadtest_result.avg_response_time))
                }
            }
        }
    }))
}

fn server(req: &Request<Body>) -> Result<Response<Vec<u8>>> {
    let body = root_page()?;
    // Send the string as a successful response.
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html; charset=utf-8")
        .body(body.as_bytes().to_owned())?;
    Ok(resp)
}

fn loadtest_dashboard(req: &Request<Body>) -> Response<Vec<u8>> {
    match server(req) {
        Ok(resp) => resp,
        Err(e) => {
            let body = format!("Loadtest Dashboard (Beta) Error: {:?}", e);
            Response::builder()
                .status(500)
                .body(body.as_bytes().to_owned())
                .unwrap()
        }
    }
}

#[fastly::main]
fn main(req: Request<Body>) -> Result<impl ResponseExt> {
    Ok(loadtest_dashboard(&req))
}
