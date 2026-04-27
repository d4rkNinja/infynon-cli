const EXIT_FLOW_FAILED: i32 = 20;
const EXIT_FLOW_INPUT_REQUIRED: i32 = 21;
const EXIT_FLOW_INVALID: i32 = 22;

struct FlowRunFailure {
    flow_id: String,
    flow_name: String,
    code: i32,
    message: String,
}

struct FlowRunRecord {
    flow_id: String,
    flow_name: String,
    exit_code: i32,
    result: Option<crate::api::types::FlowRunResult>,
    error: Option<String>,
}

pub fn cmd_flow_run(
    id: &str,
    base_url_override: Option<&str>,
    set_vars: &[(String, String)],
    format: Option<&str>,
    output: Option<&str>,
    no_input: bool,
) -> i32 {
    let human_output = format.is_none();
    match run_flow_once(id, base_url_override, set_vars, no_input, human_output) {
        Ok(result) => {
            if let Err(e) = storage::save_run_result(&result) {
                Logger::error(&format!("Could not save run result: {}", e));
            }
            if let Some(report_format) = output {
                save_run_report_fixed(&result, report_format);
            }

            let exit_code = classify_flow_result(&result);
            if let Some(stdout_format) = format {
                render_single_record(
                    &FlowRunRecord {
                        flow_id: result.flow_id.clone(),
                        flow_name: result.flow_name.clone(),
                        exit_code,
                        result: Some(result),
                        error: None,
                    },
                    stdout_format,
                );
            }
            exit_code
        }
        Err(failure) => {
            if let Some(stdout_format) = format {
                render_single_record(
                    &FlowRunRecord {
                        flow_id: failure.flow_id,
                        flow_name: failure.flow_name,
                        exit_code: failure.code,
                        result: None,
                        error: Some(failure.message),
                    },
                    stdout_format,
                );
            } else {
                Logger::error(&failure.message);
            }
            failure.code
        }
    }
}

pub fn save_run_report_pub(result: &crate::api::types::FlowRunResult, format: &str) {
    save_run_report_fixed(result, format);
}

fn run_flow_once(
    id: &str,
    base_url_override: Option<&str>,
    set_vars: &[(String, String)],
    no_input: bool,
    human_output: bool,
) -> Result<crate::api::types::FlowRunResult, FlowRunFailure> {
    let flow = storage::load_flow(id).map_err(|e| FlowRunFailure {
        flow_id: id.to_string(),
        flow_name: id.to_string(),
        code: EXIT_FLOW_INVALID,
        message: e,
    })?;
    let nodes = storage::load_nodes_map();
    let base_url = base_url_override
        .map(|s| s.to_string())
        .or_else(|| flow.base_url.clone())
        .or_else(super::env::env_base_url)
        .ok_or_else(|| FlowRunFailure {
            flow_id: flow.id.clone(),
            flow_name: flow.name.clone(),
            code: EXIT_FLOW_INPUT_REQUIRED,
            message: "BASE_URL is not set. Add it to .infynon/.env or pass --base-url <url>"
                .to_string(),
        })?;
    let initial_context = variables::parse_set_vars(set_vars);

    if human_output {
        print_flow_run_header(&flow, &base_url, &initial_context);
    }

    let on_prompt: Box<crate::api::executor::PromptCallback> = if no_input {
        Box::new(crate::api::commands::node::make_noninteractive_prompt())
    } else {
        Box::new(crate::api::commands::node::make_cli_prompt())
    };

    let on_step = if human_output {
        Some(Box::new(|step: &crate::api::types::StepResult| {
            let icon = if step.passed { "PASS" } else { "FAIL" };
            let _status = step
                .status_code
                .map(|s| s.to_string())
                .unwrap_or_else(|| "ERR".to_string());
            println!(
                "  {}  {} {} {}  {}ms",
                icon, step.node_id, step.method, step.url, step.duration_ms,
            );
            for ar in &step.assertion_results {
                if !ar.passed {
                    println!("     FAIL  {} -> actual: {}", ar.check, ar.actual);
                }
            }
            if let Some(err) = &step.error {
                println!("     Error: {}", err);
            }
        }) as Box<dyn Fn(&crate::api::types::StepResult)>)
    } else {
        None
    };

    let result = execute_flow(
        &flow,
        &nodes,
        FlowExecuteOptions {
            base_url,
            initial_context,
            on_step,
            on_prompt: Some(on_prompt),
        },
    );

    if human_output {
        print_flow_run_summary(&result);
    }

    Ok(result)
}

fn print_flow_run_header(
    flow: &Flow,
    base_url: &str,
    initial_context: &std::collections::HashMap<String, serde_json::Value>,
) {
    println!();
    Logger::title(&format!("Running flow: {}", flow.id), "cyan");

    if !initial_context.is_empty() {
        println!();
        println!(
            "  ->  Seeded {} context variable(s):",
            initial_context.len()
        );
        for key in initial_context.keys() {
            println!("     - {}", key);
        }
    }

    println!();
    println!("  ->  Target: {}", base_url);
    println!("  ->  Nodes:  {}", flow.all_node_ids().len());
    println!();
}

fn print_flow_run_summary(result: &crate::api::types::FlowRunResult) {
    let overall = if result.passed { "PASS" } else { "FAIL" };
    println!();
    println!(
        "  {}  {}/{} steps passed  ({}ms total)",
        overall,
        result.passed_count(),
        result.steps.len(),
        result.duration_ms()
    );
    println!();
}

fn classify_flow_result(result: &crate::api::types::FlowRunResult) -> i32 {
    if result.passed {
        return 0;
    }
    if result.steps.iter().any(|step| {
        step.error
            .as_deref()
            .map(is_runtime_input_error)
            .unwrap_or(false)
    }) {
        return EXIT_FLOW_INPUT_REQUIRED;
    }
    if result.steps.iter().any(|step| {
        step.error
            .as_deref()
            .map(is_flow_definition_error)
            .unwrap_or(false)
    }) {
        return EXIT_FLOW_INVALID;
    }
    EXIT_FLOW_FAILED
}

fn is_runtime_input_error(message: &str) -> bool {
    message.starts_with("Missing required runtime input") || message.contains("BASE_URL is not set")
}

fn is_flow_definition_error(message: &str) -> bool {
    message.contains("not found in library") || message.contains("Unsupported HTTP method")
}

fn render_single_record(record: &FlowRunRecord, format: &str) {
    match format.trim().to_ascii_lowercase().as_str() {
        "json" => {
            crate::utils::print_json_pretty(&record_to_json(record, "infynon.weave.run.v1"));
        }
        "markdown" => println!("{}", record_to_markdown(record)),
        "junit" => println!("{}", record_to_junit(record)),
        other => Logger::error(&format!(
            "Unsupported format '{}'. Use json | markdown | junit.",
            other
        )),
    }
}

fn render_run_suite(records: &[FlowRunRecord], format: &str) {
    match format.trim().to_ascii_lowercase().as_str() {
        "json" => {
            let passed = records
                .iter()
                .filter(|record| record.exit_code == 0)
                .count();
            let payload = serde_json::json!({
                "schema_version": "infynon.weave.run_all.v1",
                "status": if records.iter().all(|record| record.exit_code == 0) { "passed" } else { "failed" },
                "summary": {
                    "total_flows": records.len(),
                    "passed_flows": passed,
                    "failed_flows": records.len().saturating_sub(passed),
                },
                "results": records
                    .iter()
                    .map(|record| record_to_json(record, "infynon.weave.run.v1"))
                    .collect::<Vec<_>>(),
            });
            crate::utils::print_json_pretty(&payload);
        }
        "markdown" => {
            let mut out = String::new();
            let passed = records
                .iter()
                .filter(|record| record.exit_code == 0)
                .count();
            out.push_str("# Weave Flow Run Suite\n\n");
            out.push_str(&format!(
                "- Total flows: {}\n- Passed flows: {}\n- Failed flows: {}\n\n",
                records.len(),
                passed,
                records.len().saturating_sub(passed)
            ));
            for record in records {
                out.push_str(&record_to_markdown(record));
                out.push_str("\n\n");
            }
            print!("{}", out.trim_end());
        }
        "junit" => {
            let suites = records
                .iter()
                .map(record_to_junit_testsuite)
                .collect::<Vec<_>>()
                .join("");
            println!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?><testsuites>{}</testsuites>",
                suites
            );
        }
        other => Logger::error(&format!(
            "Unsupported format '{}'. Use json | markdown | junit.",
            other
        )),
    }
}

fn suite_exit_code(records: &[FlowRunRecord]) -> i32 {
    records
        .iter()
        .fold(0, |current, record| match (current, record.exit_code) {
            (EXIT_FLOW_INVALID, _) | (_, EXIT_FLOW_INVALID) => EXIT_FLOW_INVALID,
            (EXIT_FLOW_INPUT_REQUIRED, _) | (_, EXIT_FLOW_INPUT_REQUIRED) => {
                EXIT_FLOW_INPUT_REQUIRED
            }
            (EXIT_FLOW_FAILED, _) | (_, EXIT_FLOW_FAILED) => EXIT_FLOW_FAILED,
            _ => 0,
        })
}

fn record_to_json(record: &FlowRunRecord, schema_version: &str) -> serde_json::Value {
    match &record.result {
        Some(result) => serde_json::json!({
            "schema_version": schema_version,
            "status": if record.exit_code == 0 { "passed" } else { "failed" },
            "exit_code": record.exit_code,
            "flow_id": result.flow_id,
            "flow_name": result.flow_name,
            "base_url": result.base_url,
            "duration_ms": result.duration_ms(),
            "summary": {
                "total_steps": result.steps.len(),
                "passed_steps": result.passed_count(),
                "failed_steps": result.failed_count(),
            },
            "steps": result.steps,
            "final_context": result.final_context,
        }),
        None => serde_json::json!({
            "schema_version": schema_version,
            "status": "error",
            "exit_code": record.exit_code,
            "flow_id": record.flow_id,
            "flow_name": record.flow_name,
            "error": record.error.as_deref().unwrap_or("Unknown flow error"),
        }),
    }
}

fn record_to_markdown(record: &FlowRunRecord) -> String {
    match &record.result {
        Some(result) => build_run_markdown(result),
        None => format!(
            "# Flow Run: {}\n\n- Status: FAILED\n- Exit code: {}\n- Error: {}\n",
            record.flow_id,
            record.exit_code,
            record.error.as_deref().unwrap_or("Unknown flow error")
        ),
    }
}

fn record_to_junit(record: &FlowRunRecord) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>{}",
        record_to_junit_testsuite(record)
    )
}

fn record_to_junit_testsuite(record: &FlowRunRecord) -> String {
    match &record.result {
        Some(result) => {
            let failures = result.steps.iter().filter(|step| !step.passed).count();
            let cases = result
                .steps
                .iter()
                .map(|step| {
                    let mut case = format!(
                        "<testcase name=\"{}\" classname=\"{}\" time=\"{}\">",
                        xml_escape(&step.node_id),
                        xml_escape(&result.flow_id),
                        step.duration_ms as f64 / 1000.0
                    );
                    if !step.passed {
                        let message = step
                            .error
                            .clone()
                            .unwrap_or_else(|| "Assertion failure".to_string());
                        case.push_str(&format!(
                            "<failure message=\"{}\">{}</failure>",
                            xml_escape(&message),
                            xml_escape(&message)
                        ));
                    }
                    case.push_str("</testcase>");
                    case
                })
                .collect::<Vec<_>>()
                .join("");
            format!(
                "<testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" errors=\"0\" time=\"{}\">{}</testsuite>",
                xml_escape(&result.flow_id),
                result.steps.len(),
                failures,
                result.duration_ms() as f64 / 1000.0,
                cases
            )
        }
        None => format!(
            "<testsuite name=\"{}\" tests=\"1\" failures=\"1\" errors=\"0\" time=\"0\"><testcase name=\"setup\" classname=\"{}\"><failure message=\"{}\">{}</failure></testcase></testsuite>",
            xml_escape(&record.flow_id),
            xml_escape(&record.flow_id),
            xml_escape(record.error.as_deref().unwrap_or("Unknown flow error")),
            xml_escape(record.error.as_deref().unwrap_or("Unknown flow error")),
        ),
    }
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn save_run_report_fixed(result: &crate::api::types::FlowRunResult, format: &str) {
    let md = build_run_markdown(result);
    let ts = result.started_at.format("%Y%m%d-%H%M%S");
    std::fs::create_dir_all("reports").ok();

    match format.to_lowercase().as_str() {
        "markdown" | "md" => {
            let safe_flow_id = crate::utils::safe_file_stem(&result.flow_id);
            let path = format!("reports/{}-{}.md", safe_flow_id, ts);
            std::fs::write(&path, &md).ok();
            println!("  Report saved: {}", path);
        }
        "pdf" => {
            let safe_flow_id = crate::utils::safe_file_stem(&result.flow_id);
            let path = format!("reports/{}-{}.pdf", safe_flow_id, ts);
            if let Err(e) = write_run_pdf(result, &path) {
                Logger::error(&format!("Could not save PDF report: {}", e));
            } else {
                println!("  Report saved: {}", path);
            }
        }
        "both" => {
            let safe_flow_id = crate::utils::safe_file_stem(&result.flow_id);
            let md_path = format!("reports/{}-{}.md", safe_flow_id, ts);
            let pdf_path = format!("reports/{}-{}.pdf", safe_flow_id, ts);
            std::fs::write(&md_path, &md).ok();
            if let Err(e) = write_run_pdf(result, &pdf_path) {
                Logger::error(&format!(
                    "Markdown saved to {}, but PDF generation failed: {}",
                    md_path, e
                ));
            } else {
                println!("  Reports saved: {} and {}", md_path, pdf_path);
            }
        }
        _ => {}
    }
}

fn write_run_pdf(result: &crate::api::types::FlowRunResult, path: &str) -> Result<(), String> {
    let (doc, page1, layer1) =
        PdfDocument::new("INFYNON Flow Run Report", Mm(210.0), Mm(297.0), "Page 1");
    let regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| e.to_string())?;
    let bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| e.to_string())?;

    let mut page = page1;
    let mut layer = layer1;
    let mut page_num = 1usize;
    let mut y = 272.0;

    write_pdf_header(result, &doc, page, layer, &bold);

    for line in build_run_report_lines(result) {
        if y < 18.0 {
            (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), format!("Page {}", page_num + 1));
            page_num += 1;
            y = 272.0;
            write_pdf_header(result, &doc, page, layer, &bold);
        }

        doc.get_page(page)
            .get_layer(layer)
            .use_text(line, 10.5, Mm(14.0), Mm(y), &regular);
        y -= 6.0;
    }

    let file = File::create(path).map_err(|e| e.to_string())?;
    doc.save(&mut BufWriter::new(file))
        .map_err(|e| e.to_string())
}

fn write_pdf_header(
    result: &crate::api::types::FlowRunResult,
    doc: &PdfDocumentReference,
    page: PdfPageIndex,
    layer: PdfLayerIndex,
    bold: &printpdf::IndirectFontRef,
) {
    let pdf_layer = doc.get_page(page).get_layer(layer);
    pdf_layer.use_text("INFYNON Flow Run Report", 18.0, Mm(14.0), Mm(286.0), bold);
    pdf_layer.use_text(
        format!("Flow: {}", result.flow_name),
        11.0,
        Mm(14.0),
        Mm(279.0),
        bold,
    );
}

fn build_run_report_lines(result: &crate::api::types::FlowRunResult) -> Vec<String> {
    let mut lines = vec![
        format!("Flow ID: {}", result.flow_id),
        format!(
            "Started: {}",
            result.started_at.format("%Y-%m-%d %H:%M:%S UTC")
        ),
        format!("Duration: {}ms", result.duration_ms()),
        format!(
            "Status: {}",
            if result.passed { "PASSED" } else { "FAILED" }
        ),
        String::new(),
        "Steps".to_string(),
    ];

    for step in &result.steps {
        lines.push(format!(
            "- {} {} [{}] {}ms {}",
            step.method,
            step.url,
            step.status_code
                .map(|code| code.to_string())
                .unwrap_or_else(|| "ERR".to_string()),
            step.duration_ms,
            if step.passed { "PASS" } else { "FAIL" }
        ));
        if let Some(err) = &step.error {
            lines.push(format!("  Error: {}", err));
        }
        for assertion in &step.assertion_results {
            lines.push(format!(
                "  Assertion: {} => {}",
                assertion.check,
                if assertion.passed { "PASS" } else { "FAIL" }
            ));
        }
        if !step.extracted.is_empty() {
            lines.push(format!("  Extracted values: {}", step.extracted.len()));
        }
    }

    lines
}

fn build_run_markdown(result: &crate::api::types::FlowRunResult) -> String {
    let mut md = String::new();
    md.push_str(&format!("# Flow Run: {}\n\n", result.flow_id));
    md.push_str(&format!(
        "- Started: {}\n",
        result.started_at.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    md.push_str(&format!("- Duration: {}ms\n", result.duration_ms()));
    md.push_str(&format!(
        "- Status: {}\n\n",
        if result.passed { "PASSED" } else { "FAILED" }
    ));
    md.push_str("## Steps\n\n");

    for step in &result.steps {
        md.push_str(&format!(
            "### {} - {} {}\n\n",
            step.node_id, step.method, step.url
        ));
        md.push_str(&format!(
            "- Status: {}\n",
            step.status_code
                .map(|s| s.to_string())
                .unwrap_or_else(|| "ERR".to_string())
        ));
        md.push_str(&format!("- Duration: {}ms\n", step.duration_ms));
        md.push_str(&format!(
            "- Result: {}\n",
            if step.passed { "PASSED" } else { "FAILED" }
        ));

        if !step.assertion_results.is_empty() {
            md.push_str("\n**Assertions:**\n\n");
            for ar in &step.assertion_results {
                let icon = if ar.passed { "PASS" } else { "FAIL" };
                md.push_str(&format!(
                    "- {} `{}` (actual: `{}`)\n",
                    icon, ar.check, ar.actual
                ));
            }
        }

        if !step.extracted.is_empty() {
            md.push_str("\n**Extracted:**\n\n");
            for (k, v) in &step.extracted {
                md.push_str(&format!("- `{}` = `{}`\n", k, v));
            }
        }

        if let Some(err) = &step.error {
            md.push_str(&format!("\n**Error:** {}\n", err));
        }

        md.push('\n');
    }

    md
}
