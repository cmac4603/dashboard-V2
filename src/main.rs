//! Loadtest-Dashboard delivered on the edge!
#[macro_use] extern crate anyhow;
use anyhow::{Error, Result};
extern crate burden_shared;
use fastly::http::{HeaderValue, Method, StatusCode};
use fastly::request::CacheOverride;
use fastly::{Body, Request, RequestExt, Response, ResponseExt};
#[macro_use] extern crate horrorshow;
use horrorshow::prelude::*;
use horrorshow::helper::doctype;
#[macro_use] extern crate lazy_static;
use regex::Regex;

mod burden;
mod url;

use burden::BurdenRequest;


fn loadtest_result(test_name: &str, running_id: &str) -> Result<String> {
    let loadtest_result = BurdenRequest::new(test_name, running_id)?.get_response()?;
    url::loadtest_result_dashboard(test_name, running_id, loadtest_result)
}

lazy_static! {
    // /loadtests/5-minute-burden/results/1598637064
    // ^/loadtests/(?P<test_name>[\d\w-]+)/results/(?P<running_id>[\d]+)(?|$)
    static ref LOADTEST_RESULT_URL: Regex = Regex::new(
        r#"^/loadtests/(?P<test_name>[\d\w-]+)/results/(?P<running_id>[\d]+)(/|$)"#
    ).expect("Regex for loadtest results page failed to compile.");
}

fn server(req: &Request<Body>) -> Result<Response<Vec<u8>>> {
    let path = req.uri().path();
    let (body, status): (String, u16) = if LOADTEST_RESULT_URL.is_match(path) {
        let caps = LOADTEST_RESULT_URL.captures(path).unwrap();
        (loadtest_result(&caps["test_name"], &caps["running_id"])?, 200)
    } else {
        // fallback 404 page if no matches
        (format!("{}", html! {
            : doctype::HTML;
            html {
                head { title : "Loadtest Not Found"; }
                body { h1 { : Raw("Not Found") } }
            }
        }), 404)
    };

    // Send the string as a successful response.
    let resp = Response::builder()
        .status(status)
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
