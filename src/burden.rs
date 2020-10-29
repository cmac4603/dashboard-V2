// use std::error::Error;

use anyhow::{Error, Result};
use fastly::dictionary::Dictionary;
use fastly::{Body, PendingRequest, Response, Request, RequestExt};
use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;

// https://loadtesting.hearstapps.com/loadtests/5-minute-burden/results/1598637064
// https://burden.kubeprod.hearstapps.com/v2/loadtests/5-minute-jam/results/1597307460

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

        // Send it asynchronously:
        let pending = req.send_async("burden_kubeprod").map_err(Error::msg)?;
        Ok(BurdenRequest {
            loadtest_name: loadtest_name.to_string(),
            running_id: running_id.to_string(),
            pending,
        })
    }

    pub(crate) fn get_response(self) -> Result<BurdenLoadtestResult> {
        let resp = self.pending.wait().map_err(Error::msg)?;
        if resp.status() == 200 {
            BurdenLoadtestResult::new(resp)
        } else {
            let err_body = std::str::from_utf8(resp.into_body().into_bytes().as_ref())?;
            Err(anyhow!("Error retrieving loadtest result from burden"))
        }
    }
}

#[derive(Clone, Deserialize)]
pub(crate) struct BurdenLoadtestResult {
    pub(crate) avg_bytes: f64,
    pub(crate) avg_response_time: f64,
    pub(crate) avg_throughput: f64,
    pub(crate) duration: String, // grr
    pub(crate) errors_count: f64,
    pub(crate) name: String, // format: {loadtest_name}-{running_id}
    pub(crate) ninety_line: f64,
    pub(crate) threshold_criteria: Vec<String>,
    pub(crate) threshold_results: Vec<ThresholdResult>,
    pub(crate) version_tag: String,
}

#[derive(Clone, Deserialize)]
pub(crate) struct ThresholdResult {
    pub(crate) field: String,
    pub(crate) success: bool,
    pub(crate) value: f64,
}

impl BurdenLoadtestResult {
    pub(crate) fn new(response: Response<Body>) -> Result<Self> {
        let json: Vec<BurdenLoadtestResult> = serde_json::from_slice(response.into_body().into_bytes().as_ref())?;
        let result = json[0].clone();
        Ok(result)
    }
}
