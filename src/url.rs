use anyhow::Result;
use serde::Serialize;
use tera::{Context, Tera};

use burden_shared::dynamo_db::constructor::LoadtestResult as BurdenLoadtestResult;

static LOADTEST_RESULTS_DASHBOARD: &'static str = include_str!("templates/loadtest_dashboard.html.j2");

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("loadtest_results_dashboard", LOADTEST_RESULTS_DASHBOARD),
        ]).unwrap();
        tera
    };
}

#[derive(Serialize)]
struct TopStats {
    avg_bytes: f64,
    avg_response_time: String,
    avg_throughput: String,
    errors_count: String,
    ninety_line: String,
}


pub(crate) fn loadtest_result_dashboard(test_name: &str,running_id: &str, loadtest_result: BurdenLoadtestResult) -> Result<String> {
    let mut ctx = Context::new();
    ctx.insert("dashboard_uid", &2961);
    // ctx.insert("end_time", &self.end_time);
    // ctx.insert("reason", &self.reason);
    ctx.insert("running_id", running_id);
    // ctx.insert("start_time", &self.start_time);
    ctx.insert("status", &"Ended");
    ctx.insert("testname", &test_name.replace("-", "_"));
    ctx.insert("url_logs", &"");
    ctx.insert("test_logs", &"");

    let top_stats = TopStats {
        avg_bytes: loadtest_result.avg_bytes.abs(),
        avg_response_time: format!("{:.2}", loadtest_result.avg_response_time),
        avg_throughput: format!("{:.2}", loadtest_result.avg_throughput),
        errors_count: format!("{:.2}", loadtest_result.errors_count),
        ninety_line: format!("{:.2}", loadtest_result.ninety_line),
    };
    ctx.insert("top_stats", &top_stats);
    let loadtest_dashboard = TEMPLATES.render("loadtest_results_dashboard", &ctx)?;
    Ok(loadtest_dashboard)
}
