use crate::cli::scan;

pub fn cmd_fix_auto(pkg_file: Option<&str>) {
    // Delegate to the existing scan + auto-fix pipeline
    scan::run_scan(None, Some(scan::FixLevel::All), pkg_file);
}
