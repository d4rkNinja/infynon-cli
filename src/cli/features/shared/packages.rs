use crate::engine::scanner;
use dialoguer::Select;

pub(crate) fn load_packages(explicit_file: Option<&str>) -> Vec<scanner::LockedPackage> {
    if let Some(file) = explicit_file {
        return scanner::detect_locked_packages(Some(file));
    }

    let found = scanner::detect_lock_files();
    if found.is_empty() {
        return vec![];
    }
    if found.len() == 1 {
        return scanner::parse_selected_files(&[found[0].0]);
    }

    println!();
    let mut options = vec![format!("  ✦ All ({} files detected)", found.len())];
    options.extend(
        found
            .iter()
            .map(|(file, eco)| format!("  {}  ({})", file, eco)),
    );
    let selection = Select::new()
        .with_prompt("Multiple lock files detected — select which to scan")
        .items(&options)
        .default(0)
        .interact_opt()
        .ok()
        .flatten();

    match selection {
        Some(0) | None => {
            let files: Vec<&str> = found.iter().map(|(file, _)| *file).collect();
            scanner::parse_selected_files(&files)
        }
        Some(index) => scanner::parse_selected_files(&[found[index - 1].0]),
    }
}
