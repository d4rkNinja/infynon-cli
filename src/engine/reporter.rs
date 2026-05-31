use crate::engine::osv::OsvVulnDetail;
use crate::engine::scanner::LockedPackage;
use printpdf::*;
use std::fs;
use std::io::BufWriter;

pub struct ScanFinding {
    pub package: LockedPackage,
    pub vuln: OsvVulnDetail,
    pub severity: &'static str,
    pub fixed_version: Option<String>,
    /// Latest stable version from registry — used when no fix exists in vulnerability DB.
    pub suggested_version: Option<String>,
}

/// Write a Markdown report to disk.
pub fn write_markdown(findings: &[ScanFinding], path: &str) -> Result<(), String> {
    let mut out = String::new();
    out.push_str("# 🛡️ INFYNON Security Scan Report\n\n");
    out.push_str(&format!("> Generated: {}  \n", sys_time_readable()));
    out.push_str(&format!(
        "> Total vulnerabilities: **{}**\n\n",
        findings.len()
    ));

    let (crit, high, med, low, info) = severity_counts(findings);
    out.push_str(&format!(
        "| 🔴 Critical | 🟠 High | 🟡 Medium | 🟢 Low | ℹ️ Info |\n|---|---|---|---|---|\n| {} | {} | {} | {} | {} |\n\n",
        crit, high, med, low, info
    ));
    out.push_str("---\n\n");

    // Unified vulnerability + remediation table
    out.push_str("| Risk | Package | Version | CVE / ID | Remediation | Upgrade Command |\n");
    out.push_str("|------|---------|---------|----------|-------------|------------------|\n");
    for f in findings {
        let badge = severity_badge(f.severity);
        let (fix_col, cmd_col) = if let Some(ref fv) = f.fixed_version {
            (
                format!("✅ `{}`", fv),
                format!("`{}`", upgrade_cmd(&f.package, fv)),
            )
        } else if let Some(ref sv) = f.suggested_version {
            (
                format!("⚠️ No DB fix — try `{}`", sv),
                format!("`{}`", upgrade_cmd(&f.package, sv)),
            )
        } else {
            ("❌ No fix available".to_string(), "—".to_string())
        };
        out.push_str(&format!(
            "| {} `{}` | {} | `{}` | [`{}`](https://osv.dev/vulnerability/{}) | {} | {} |\n",
            badge,
            f.severity,
            f.package.name,
            f.package.version,
            f.vuln.id,
            f.vuln.id,
            fix_col,
            cmd_col
        ));
    }
    out.push_str("\n---\n\n");

    // Detailed findings
    for f in findings {
        let badge = severity_badge(f.severity);
        out.push_str(&format!(
            "### {} {} — `{}`\n\n",
            badge, f.package.name, f.package.version
        ));
        out.push_str("| Field | Value |\n|---|---|\n");
        out.push_str(&format!("| **Ecosystem** | {} |\n", f.package.ecosystem));
        out.push_str(&format!("| **Source** | `{}` |\n", f.package.source));
        out.push_str(&format!(
            "| **CVE / ID** | [`{}`](https://osv.dev/vulnerability/{}) |\n",
            f.vuln.id, f.vuln.id
        ));
        out.push_str(&format!(
            "| **Risk Level** | {} `{}` |\n",
            badge, f.severity
        ));
        out.push_str(&format!(
            "| **Current Version** | `{}` |\n",
            f.package.version
        ));
        if let Some(ref fv) = f.fixed_version {
            out.push_str(&format!("| **Fix Version** | ✅ `{}` |\n", fv));
            out.push_str(&format!(
                "| **Upgrade Command** | `{}` |\n",
                upgrade_cmd(&f.package, fv)
            ));
        } else if let Some(ref sv) = f.suggested_version {
            out.push_str(&format!(
                "| **Fix Version** | ⚠️ No fix in DB — latest stable: `{}` |\n",
                sv
            ));
            out.push_str(&format!(
                "| **Upgrade Command** | `{}` |\n",
                upgrade_cmd(&f.package, sv)
            ));
        } else {
            out.push_str("| **Fix Version** | ❌ No fix available |\n");
        }
        out.push_str(&format!(
            "| **Published** | {} |\n",
            f.vuln.published.as_deref().unwrap_or("N/A")
        ));
        if let Some(ref s) = f.vuln.summary {
            out.push_str(&format!("\n**Summary:** {}\n\n", s));
        }
        if let Some(ref d) = f.vuln.details {
            let short: String = d.chars().take(500).collect();
            out.push_str(&format!("**Details:**\n```\n{}\n```\n\n", short));
        }
        if !f.vuln.references.is_empty() {
            out.push_str("**References:**\n\n");
            for r in &f.vuln.references {
                out.push_str(&format!(
                    "- [{}]({})\n",
                    r.kind.as_deref().unwrap_or("link"),
                    r.url
                ));
            }
        }
        out.push_str("\n---\n\n");
    }
    fs::write(path, out).map_err(|e| e.to_string())
}

/// Write a professional PDF report.
pub fn write_pdf(findings: &[ScanFinding], path: &str) -> Result<(), String> {
    let (doc, page1, layer1) = PdfDocument::new(
        "INFYNON Security Scan Report",
        Mm(210.0_f32),
        Mm(297.0_f32),
        "Page 1",
    );

    let bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| e.to_string())?;
    let reg = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| e.to_string())?;
    let italic = doc
        .add_builtin_font(BuiltinFont::HelveticaOblique)
        .map_err(|e| e.to_string())?;

    let mut ctx = PdfCtx {
        doc: &doc,
        bold: &bold,
        reg: &reg,
        italic: &italic,
        page_idx: page1,
        layer_idx: layer1,
        y: 277.0_f32,
        page_num: 1,
    };

    // ── Full-page dark background ────────────────────────────────────────
    {
        let layer = ctx.layer();
        layer.set_fill_color(Color::Rgb(Rgb::new(0.09, 0.11, 0.16, None)));
        layer.add_rect(Rect::new(Mm(0.0), Mm(0.0), Mm(210.0), Mm(297.0)));
    }

    // ── Header bar ────────────────────────────────────────────────────────
    {
        let layer = ctx.layer();
        // Dark header background
        layer.set_fill_color(Color::Rgb(Rgb::new(0.05, 0.07, 0.12, None)));
        layer.add_rect(Rect::new(Mm(0.0), Mm(277.0), Mm(210.0), Mm(297.0)));

        // Logo text
        layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.85, 0.9, None)));
        layer.use_text("INFYNON", 22.0_f32, Mm(14.0), Mm(283.0), &bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.7, 0.7, 0.8, None)));
        layer.use_text("Security Scan Report", 11.0_f32, Mm(14.0), Mm(279.0), &reg);

        // Right-side meta
        layer.set_fill_color(Color::Rgb(Rgb::new(0.6, 0.6, 0.7, None)));
        layer.use_text(
            format!("Generated: {}", sys_time_readable()),
            8.0_f32,
            Mm(130.0),
            Mm(283.0),
            &reg,
        );
        layer.use_text(
            format!("Total Vulnerabilities: {}", findings.len()),
            8.0_f32,
            Mm(130.0),
            Mm(279.5),
            &reg,
        );
    }
    ctx.y = 270.0;

    // ── Severity summary bar ──────────────────────────────────────────────
    ctx.y -= 4.0;
    {
        let (crit, high, med, low, info) = severity_counts(findings);
        let layer = ctx.layer();

        // Section bg
        layer.set_fill_color(Color::Rgb(Rgb::new(0.08, 0.10, 0.16, None)));
        layer.add_rect(Rect::new(
            Mm(10.0),
            Mm(ctx.y - 6.0),
            Mm(200.0),
            Mm(ctx.y + 2.0),
        ));

        let items = [
            ("CRITICAL", crit, Rgb::new(0.9, 0.15, 0.15, None), 14.0_f32),
            ("HIGH", high, Rgb::new(0.95, 0.55, 0.05, None), 42.0_f32),
            ("MEDIUM", med, Rgb::new(0.9, 0.8, 0.1, None), 66.0_f32),
            ("LOW", low, Rgb::new(0.2, 0.85, 0.3, None), 92.0_f32),
            ("INFO", info, Rgb::new(0.4, 0.6, 0.95, None), 114.0_f32),
        ];
        for &(label, count, ref color, x) in &items {
            layer.set_fill_color(Color::Rgb(color.clone()));
            layer.use_text(
                format!("{}: {}", label, count),
                8.0_f32,
                Mm(x),
                Mm(ctx.y - 1.5),
                &bold,
            );
        }
        layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None))); // reset
    }
    ctx.y -= 12.0;

    // ── Per-finding blocks ────────────────────────────────────────────────
    for (i, f) in findings.iter().enumerate() {
        ctx.ensure_space(50.0)?;

        let sev_color: Rgb = match f.severity {
            "CRITICAL" => Rgb::new(0.9, 0.15, 0.15, None),
            "HIGH" => Rgb::new(0.95, 0.55, 0.05, None),
            "MEDIUM" => Rgb::new(0.9, 0.80, 0.1, None),
            "LOW" => Rgb::new(0.2, 0.85, 0.3, None),
            _ => Rgb::new(0.5, 0.55, 0.70, None),
        };

        // Finding header strip
        {
            let layer = ctx.layer();
            layer.set_fill_color(Color::Rgb(Rgb::new(0.08, 0.10, 0.17, None)));
            layer.add_rect(Rect::new(
                Mm(10.0),
                Mm(ctx.y - 5.5),
                Mm(200.0),
                Mm(ctx.y + 1.5),
            ));

            // Severity badge — wide enough for "INFORMATIONAL" (~13 chars at 7pt ≈ 21mm)
            layer.set_fill_color(Color::Rgb(sev_color.clone()));
            layer.add_rect(Rect::new(
                Mm(10.0),
                Mm(ctx.y - 5.5),
                Mm(36.0),
                Mm(ctx.y + 1.5),
            ));
            layer.set_fill_color(Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None)));
            layer.use_text(f.severity, 7.0_f32, Mm(11.5), Mm(ctx.y - 2.5), &bold);

            // Package name — starts after badge with clear gap
            layer.set_fill_color(Color::Rgb(Rgb::new(0.95, 0.95, 1.0, None)));
            layer.use_text(
                format!("{}. {} @ {}", i + 1, f.package.name, f.package.version),
                10.0_f32,
                Mm(39.0),
                Mm(ctx.y - 2.5),
                &bold,
            );
            // CVE ID right-aligned
            layer.set_fill_color(Color::Rgb(Rgb::new(0.4, 0.7, 1.0, None)));
            layer.use_text(&f.vuln.id, 8.0_f32, Mm(155.0), Mm(ctx.y - 2.5), &italic);
        }
        ctx.y -= 9.0;

        // Fields grid
        let fields: Vec<(&str, String)> = {
            let mut v = vec![
                ("Ecosystem", f.package.ecosystem.clone()),
                (
                    "Published",
                    f.vuln.published.as_deref().unwrap_or("N/A").to_string(),
                ),
            ];
            if let Some(ref fv) = f.fixed_version {
                v.push(("Fix Version", fv.clone()));
                v.push(("Upgrade", upgrade_cmd(&f.package, fv)));
            } else if let Some(ref sv) = f.suggested_version {
                v.push(("Remediation", format!("No DB fix — try latest: {}", sv)));
                v.push(("Upgrade", upgrade_cmd(&f.package, sv)));
            } else {
                v.push(("Remediation", "No fix available".to_string()));
            }
            v
        };

        // Two-column field layout
        for chunk in fields.chunks(2) {
            ctx.ensure_space(8.0)?;
            let layer = ctx.layer();
            for (col, (label, val)) in chunk.iter().enumerate() {
                let x = if col == 0 { 12.0_f32 } else { 110.0_f32 };
                layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.55, 0.7, None)));
                layer.use_text(format!("{}:", label), 7.5_f32, Mm(x), Mm(ctx.y), &bold);
                layer.set_fill_color(Color::Rgb(Rgb::new(0.85, 0.85, 0.9, None)));
                let val_short: String = val.chars().take(45).collect();
                layer.use_text(&val_short, 7.5_f32, Mm(x + 22.0), Mm(ctx.y), &reg);
            }
            ctx.y -= 6.0;
        }

        // Summary line
        if let Some(ref s) = f.vuln.summary {
            ctx.ensure_space(10.0)?;
            let short: String = s.chars().take(120).collect();
            let layer = ctx.layer();
            layer.set_fill_color(Color::Rgb(Rgb::new(0.45, 0.5, 0.65, None)));
            layer.use_text("Summary:", 7.5_f32, Mm(12.0), Mm(ctx.y), &bold);
            layer.set_fill_color(Color::Rgb(Rgb::new(0.78, 0.78, 0.85, None)));
            layer.use_text(&short, 7.5_f32, Mm(34.0), Mm(ctx.y), &reg);
            ctx.y -= 6.0;
        }

        // Divider
        {
            ctx.ensure_space(5.0)?;
            let layer = ctx.layer();
            layer.set_fill_color(Color::Rgb(Rgb::new(0.15, 0.18, 0.25, None)));
            layer.add_rect(Rect::new(
                Mm(10.0),
                Mm(ctx.y + 1.0),
                Mm(200.0),
                Mm(ctx.y + 1.3),
            ));
        }
        ctx.y -= 6.0;
    }

    // ── Footer ────────────────────────────────────────────────────────────
    {
        let layer = ctx.layer();
        layer.set_fill_color(Color::Rgb(Rgb::new(0.08, 0.09, 0.13, None)));
        layer.add_rect(Rect::new(Mm(0.0), Mm(0.0), Mm(210.0), Mm(9.0)));
        layer.set_fill_color(Color::Rgb(Rgb::new(0.4, 0.4, 0.55, None)));
        layer.use_text(
            "INFYNON Security — Vulnerability Intelligence",
            7.0_f32,
            Mm(14.0),
            Mm(3.0),
            &reg,
        );
        layer.use_text("CONFIDENTIAL", 7.0_f32, Mm(170.0), Mm(3.0), &bold);
    }

    let file = fs::File::create(path).map_err(|e| e.to_string())?;
    doc.save(&mut BufWriter::new(file))
        .map_err(|e| e.to_string())
}

// ── PDF context helper ─────────────────────────────────────────────────────────

struct PdfCtx<'a> {
    doc: &'a PdfDocumentReference,
    bold: &'a IndirectFontRef,
    reg: &'a IndirectFontRef,
    italic: &'a IndirectFontRef,
    page_idx: PdfPageIndex,
    layer_idx: PdfLayerIndex,
    y: f32,
    page_num: usize,
}

impl<'a> PdfCtx<'a> {
    fn layer(&self) -> PdfLayerReference {
        self.doc.get_page(self.page_idx).get_layer(self.layer_idx)
    }

    fn ensure_space(&mut self, needed: f32) -> Result<(), String> {
        if self.y < needed + 12.0 {
            let (new_page, new_layer) = self.doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
            self.page_idx = new_page;
            self.layer_idx = new_layer;
            self.y = 285.0;
            self.page_num += 1;

            // Full-page dark background for new page
            let layer = self.layer();
            layer.set_fill_color(Color::Rgb(Rgb::new(0.09, 0.11, 0.16, None)));
            layer.add_rect(Rect::new(Mm(0.0), Mm(0.0), Mm(210.0), Mm(297.0)));

            // Page header continuation
            layer.set_fill_color(Color::Rgb(Rgb::new(0.05, 0.07, 0.12, None)));
            layer.add_rect(Rect::new(Mm(0.0), Mm(288.0), Mm(210.0), Mm(297.0)));
            layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.85, 0.9, None)));
            layer.use_text("INFYNON", 14.0, Mm(14.0), Mm(291.0), self.bold);
            layer.set_fill_color(Color::Rgb(Rgb::new(0.6, 0.6, 0.7, None)));
            layer.use_text(
                format!("Security Scan Report — continued (page {})", self.page_num),
                8.0,
                Mm(42.0),
                Mm(291.5),
                self.reg,
            );
        }
        Ok(())
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────────

pub fn severity_counts(findings: &[ScanFinding]) -> (usize, usize, usize, usize, usize) {
    let (mut crit, mut high, mut med, mut low, mut info) = (0, 0, 0, 0, 0);
    for f in findings {
        match f.severity {
            "CRITICAL" => crit += 1,
            "HIGH" => high += 1,
            "MEDIUM" => med += 1,
            "LOW" => low += 1,
            _ => info += 1,
        }
    }
    (crit, high, med, low, info)
}

fn severity_badge(sev: &str) -> &'static str {
    match sev {
        "CRITICAL" => "🔴",
        "HIGH" => "🟠",
        "MEDIUM" => "🟡",
        "LOW" => "🟢",
        _ => "ℹ️",
    }
}

fn upgrade_cmd(pkg: &LockedPackage, fixed: &str) -> String {
    crate::cli::scan::upgrade_cmd(pkg, fixed)
}

fn sys_time_readable() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Simple readable format from unix timestamp
    let minutes = (secs / 60) % 60;
    let hours = (secs / 3600) % 24;
    let days = secs / 86400;
    format!("Day {} {:02}:{:02} UTC", days, hours, minutes)
}
