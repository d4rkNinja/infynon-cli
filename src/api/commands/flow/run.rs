pub fn cmd_flow_run(
    id: &str,
    base_url_override: Option<&str>,
    set_vars: &[(String, String)],
    output: Option<&str>,
) {
    println!();
    Logger::title(&format!("Running flow: {}", id), "cyan");

    let flow = match storage::load_flow(id) {
        Ok(f) => f,
        Err(e) => {
            Logger::error(&e);
            return;
        }
    };

    let nodes = storage::load_nodes_map();

    let base_url = match base_url_override
        .map(|s| s.to_string())
        .or_else(|| flow.base_url.clone())
        .or_else(|| super::env::env_base_url())
    {
        Some(u) => u,
        None => {
            Logger::error("BASE_URL is not set. Add it to .infynon/.env or pass --base-url <url>");
            return;
        }
    };

    let initial_context = variables::parse_set_vars(set_vars);

    if !initial_context.is_empty() {
        println!();
        println!(
            "  {}  Seeded {} context variable(s):",
            "→".bright_cyan(),
            initial_context.len()
        );
        for k in initial_context.keys() {
            println!("     {} {}", "·".truecolor(100, 100, 140), k.bright_cyan());
        }
    }

    println!();
    println!(
        "  {}  Target: {}",
        "→".bright_cyan(),
        base_url.truecolor(160, 160, 200)
    );
    println!(
        "  {}  Nodes:  {}",
        "→".bright_cyan(),
        flow.all_node_ids().len().to_string().bright_cyan()
    );
    println!();

    let on_prompt = crate::api::commands::node::make_cli_prompt();
    let result = execute_flow(
        &flow,
        &nodes,
        FlowExecuteOptions {
            base_url: base_url.clone(),
            initial_context,
            on_step: Some(Box::new(|step| {
                let icon = if step.passed {
                    "✔".bright_green().to_string()
                } else {
                    "✘".bright_red().to_string()
                };
                let status = step
                    .status_code
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "ERR".to_string());
                println!(
                    "  {}  {} {} {}  {}ms",
                    icon,
                    step.node_id.bold(),
                    step.method.bright_yellow(),
                    step.url.truecolor(100, 100, 160),
                    step.duration_ms.to_string().truecolor(200, 200, 100),
                );
                for ar in &step.assertion_results {
                    if !ar.passed {
                        println!(
                            "     {}  {} → actual: {}",
                            "✘".bright_red(),
                            ar.check.truecolor(200, 100, 100),
                            ar.actual.truecolor(180, 100, 100),
                        );
                    }
                }
                if let Some(err) = &step.error {
                    println!(
                        "     {}  {}",
                        "Error:".bright_red(),
                        err.truecolor(220, 100, 100)
                    );
                }
            })),
            on_prompt: Some(Box::new(on_prompt)),
        },
    );

    println!();
    let overall = if result.passed {
        "✔ PASSED".bright_green().to_string()
    } else {
        "✘ FAILED".bright_red().to_string()
    };

    println!(
        "  {}  {}/{} steps passed  ({}ms total)",
        overall,
        result.passed_count(),
        result.steps.len(),
        result.duration_ms(),
    );
    println!();

    // Save run result
    if let Err(e) = storage::save_run_result(&result) {
        Logger::error(&format!("Could not save run result: {}", e));
    }

    // Save report if requested
    if let Some(fmt) = output {
        save_run_report_fixed(&result, fmt);
    }
}

pub fn save_run_report_pub(result: &crate::api::types::FlowRunResult, format: &str) {
    save_run_report_fixed(result, format);
}

fn save_run_report(result: &crate::api::types::FlowRunResult, format: &str) {
    let md = build_run_markdown(result);
    let ts = result.started_at.format("%Y%m%d-%H%M%S");
    std::fs::create_dir_all("reports").ok();

    match format.to_lowercase().as_str() {
        "markdown" | "md" => {
            let path = format!("reports/{}-{}.md", result.flow_id, ts);
            std::fs::write(&path, &md).ok();
            println!("  {}  Report saved: {}", "✔".bright_green(), path);
        }
        "pdf" => {
            let path = format!("reports/{}-{}.pdf", result.flow_id, ts);
            println!("  {}  Report saved: {}", "✔".bright_green(), path);
        }
        "both" => {
            let md_path = format!("reports/{}-{}.md", result.flow_id, ts);
            let pdf_path = format!("reports/{}-{}.pdf", result.flow_id, ts);
            std::fs::write(&md_path, &md).ok();
            println!(
                "  {}  Reports saved: {} and {}",
                "✔".bright_green(),
                md_path,
                pdf_path
            );
        }
        _ => {}
    }
}

fn save_run_report_fixed(result: &crate::api::types::FlowRunResult, format: &str) {
    let md = build_run_markdown(result);
    let ts = result.started_at.format("%Y%m%d-%H%M%S");
    std::fs::create_dir_all("reports").ok();

    match format.to_lowercase().as_str() {
        "markdown" | "md" => {
            let path = format!("reports/{}-{}.md", result.flow_id, ts);
            std::fs::write(&path, &md).ok();
            println!("  {}  Report saved: {}", "âœ”".bright_green(), path);
        }
        "pdf" => {
            let path = format!("reports/{}-{}.pdf", result.flow_id, ts);
            if let Err(e) = write_run_pdf(result, &path) {
                Logger::error(&format!("Could not save PDF report: {}", e));
            } else {
                println!("  {}  Report saved: {}", "âœ”".bright_green(), path);
            }
        }
        "both" => {
            let md_path = format!("reports/{}-{}.md", result.flow_id, ts);
            let pdf_path = format!("reports/{}-{}.pdf", result.flow_id, ts);
            std::fs::write(&md_path, &md).ok();
            if let Err(e) = write_run_pdf(result, &pdf_path) {
                Logger::error(&format!(
                    "Markdown saved to {}, but PDF generation failed: {}",
                    md_path, e
                ));
            } else {
                println!(
                    "  {}  Reports saved: {} and {}",
                    "âœ”".bright_green(),
                    md_path,
                    pdf_path
                );
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
            (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), &format!("Page {}", page_num + 1));
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
            "### {} — {} {}\n\n",
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
                let icon = if ar.passed { "✔" } else { "✘" };
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

// ── flow run all ──────────────────────────────────────────────────────────────

