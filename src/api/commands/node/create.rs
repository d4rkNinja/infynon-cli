pub fn cmd_node_create(ai_description: Option<&str>) {
    println!();
    Logger::title("INFYNON API", "cyan");

    let node = if let Some(desc) = ai_description {
        create_node_from_ai(desc)
    } else {
        create_node_interactive()
    };

    match storage::save_node(&node) {
        Ok(path) => {
            println!();
            println!("  {}  Node saved: {}", "✔".bright_green(), node.id.bold());
            println!(
                "     Path:   {}",
                path.display().to_string().truecolor(100, 100, 140)
            );
            println!("     Method: {} {}", node.method.bright_cyan(), node.path);
            println!();
        }
        Err(e) => {
            Logger::error(&format!("Failed to save node: {}", e));
        }
    }
}

fn create_node_interactive() -> Node {
    Logger::step("Creating new node (interactive)");
    println!();

    let id = prompt("  Node ID (e.g. login, create-cart): ");
    let name = prompt("  Display name: ");
    let method = prompt("  HTTP method [GET/POST/PUT/PATCH/DELETE]: ").to_uppercase();
    let method = if method.is_empty() {
        "GET".to_string()
    } else {
        method
    };
    let path = prompt("  Path (e.g. /users/{user_id}): ");

    let mut node = Node::new(&id, &name, &method, &path);

    // Headers
    println!();
    println!(
        "  {}  Add headers? (leave blank to skip)",
        "→".truecolor(100, 100, 140)
    );
    loop {
        let key = prompt("    Header name (or Enter to finish): ");
        if key.is_empty() {
            break;
        }
        let val = prompt(&format!("    {} value: ", key));
        node.headers.insert(key, val);
    }

    // Body
    if method == "POST" || method == "PUT" || method == "PATCH" {
        println!();
        let body = prompt("  JSON body (or Enter to skip, use {var} for placeholders): ");
        if !body.is_empty() {
            node.body_json = Some(body);
        }
    }

    // Extractions
    println!();
    println!(
        "  {}  Add extractions? (pull values from response into context)",
        "→".truecolor(100, 100, 140)
    );
    let ai_ext = prompt("    Auto-generate extractions? [Y/n]: ");
    if ai_ext.trim().to_lowercase() != "n" {
        node.extractions = ai::generate_extractions(&node);
        println!("    Generated {} extraction(s):", node.extractions.len());
        for e in &node.extractions {
            println!(
                "      {}  {} ← {}",
                "·".truecolor(100, 100, 140),
                e.name.bright_cyan(),
                e.from
            );
        }
    } else {
        loop {
            let name = prompt("    Variable name (or Enter to finish): ");
            if name.is_empty() {
                break;
            }
            let from = prompt(&format!(
                "    Extract '{}' from (e.g. body.token, header.location): ",
                name
            ));
            node.extractions.push(Extraction { name, from });
        }
    }

    // Assertions
    println!();
    println!("  {}  Add assertions?", "→".truecolor(100, 100, 140));
    let ai_assert = prompt("    Auto-generate assertions? [Y/n]: ");
    if ai_assert.trim().to_lowercase() != "n" {
        node.assertions = ai::generate_assertions(&node);
        println!("    Generated {} assertion(s):", node.assertions.len());
        for a in &node.assertions {
            println!(
                "      {}  {}",
                "·".truecolor(100, 100, 140),
                a.check.bright_cyan()
            );
        }
    } else {
        loop {
            let check = prompt("    Assertion (e.g. 'status == 201', or Enter to finish): ");
            if check.is_empty() {
                break;
            }
            node.assertions.push(Assertion {
                check,
                on_fail: OnFail::Stop,
                enabled: true,
            });
        }
    }

    node
}

fn create_node_from_ai(description: &str) -> Node {
    Logger::step(&format!("Generating node from: \"{}\"", description));

    // Parse description heuristically
    // Expect format: "METHOD /path" or just a description
    // Try to parse "METHOD /path [rest...]"
    let words: Vec<&str> = description.splitn(3, ' ').collect();
    let (method, path) = match words.as_slice() {
        [m, p, ..] => {
            let mu = m.to_uppercase();
            if ["GET", "POST", "PUT", "PATCH", "DELETE"].contains(&mu.as_str())
                && p.starts_with('/')
            {
                // Take only the path token (stop at first space)
                let clean_path = p.split_whitespace().next().unwrap_or(p).to_string();
                (mu, clean_path)
            } else {
                (infer_method(description), infer_path(description))
            }
        }
        _ => (infer_method(description), infer_path(description)),
    };

    // Build id from path
    let id = path_to_id(&path);
    let name = format!(
        "{} {}",
        title_case(&method.to_lowercase()),
        path.trim_matches('/')
            .replace('/', " ")
            .replace(['{', '}'], "")
    );

    let mut node = Node::new(&id, &name, &method, &path);

    // Auto-generate extractions and assertions
    node.extractions = ai::generate_extractions(&node);
    node.assertions = ai::generate_assertions(&node);

    // Add default JSON content-type for POST/PUT/PATCH
    if ["POST", "PUT", "PATCH"].contains(&method.as_str()) {
        node.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
    }

    // Add auth header placeholder if path doesn't look like auth itself
    let path_lower = path.to_lowercase();
    if !path_lower.contains("login") && !path_lower.contains("auth") {
        node.headers
            .insert("Authorization".to_string(), "Bearer {token}".to_string());
    }

    println!();
    println!("  {}  Generated node:", "✔".bright_green());
    println!("     ID:     {}", node.id.bright_cyan());
    println!("     Method: {} {}", node.method.bright_yellow(), node.path);
    println!("     Extractions: {}", node.extractions.len());
    println!("     Assertions:  {}", node.assertions.len());

    node
}

fn has_any_keyword(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|k| text.contains(k))
}

fn infer_method(desc: &str) -> String {
    let d = desc.to_lowercase();
    if has_any_keyword(&d, &["create", "add", "register", "login", "post"]) {
        "POST".to_string()
    } else if has_any_keyword(&d, &["update", "edit", "change"]) {
        "PATCH".to_string()
    } else if has_any_keyword(&d, &["delete", "remove"]) {
        "DELETE".to_string()
    } else {
        "GET".to_string()
    }
}

fn infer_path(desc: &str) -> String {
    // Look for /path-like pattern
    if let Some(pos) = desc.find('/') {
        let end = desc[pos..]
            .find([' ', '"', '\''])
            .unwrap_or(desc.len() - pos);
        return desc[pos..pos + end].to_string();
    }
    // Build from description keywords
    let keywords: Vec<&str> = desc
        .split_whitespace()
        .filter(|w| {
            ![
                "GET", "POST", "PUT", "PATCH", "DELETE", "a", "an", "the", "to", "from", "with",
            ]
            .contains(w)
        })
        .take(2)
        .collect();
    format!("/{}", keywords.join("/").to_lowercase())
}

fn path_to_id(path: &str) -> String {
    path.trim_matches('/')
        .replace('/', "-")
        .replace(['{', '}'], "")
        .to_lowercase()
}

fn title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

