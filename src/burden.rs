// use std::error::Error;

use anyhow::{Error, Result};
use burden_shared::dynamo_db::constructor::{LoadtestResult as BurdenLoadtestResult, ThresholdResult};
use burden_shared::error::BurdenError;
use fastly::dictionary::Dictionary;
use fastly::{Body, PendingRequest, Response, Request, RequestExt};
use http::status::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;

// https://loadtesting.hearstapps.com/loadtests/5-minute-burden/results/1598637064
// https://burden.kubeprod.hearstapps.com/v2/loadtests/5-minute-jam/results/1597307460
// https://hmm-loadtesting-beta.edgecompute.app/loadtests/5-minute-burden/results/1598637064

pub(crate) struct BurdenRequest {
    loadtest_name: String,
    running_id: String,
    pending: PendingRequest,
}

impl BurdenRequest {
    pub(crate) fn new(loadtest_name: &str, running_id: &str) -> Result<Self> {
        let secrets = Dictionary::open("secrets");
        let req = Request::builder()
            .method("GET")
            .header("accept", "application/json")
            .header("x-api-key", secrets.get("burden_api_key").unwrap())
            .uri(format!(
                "https://burden.kubeprod.hearstapps.com/v2/loadtests/{}/results/{}",
                loadtest_name, running_id
            )).body(vec![])?;

        let pending = req.send_async("burden_kubeprod").map_err(Error::msg)?;
        Ok(BurdenRequest {
            loadtest_name: loadtest_name.to_string(),
            running_id: running_id.to_string(),
            pending,
        })
    }


    pub(crate) fn get_response(self) -> Result<BurdenLoadtestResult> {
        // TODO: sort this rubbish out
        fn map_this_err(x: BurdenError) -> Error {
            anyhow!(x.to_string())
        }
        let resp = self.pending.wait().map_err(Error::msg)?;
        match resp.status() {
            StatusCode::OK => BurdenLoadtestResult::new(resp).map_err(map_this_err),
            StatusCode::NO_CONTENT => Ok(BurdenLoadtestResult::default()),  // test may not yet be finished
            _ => {
                let bytes = resp.into_body().into_bytes().clone();
                Err(anyhow!("Error retrieving loadtest result from burden: {}", std::str::from_utf8(bytes.as_ref())?))
            }
        }
    }
}
