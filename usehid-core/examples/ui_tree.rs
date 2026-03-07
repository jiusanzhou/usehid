//! Example: Accessibility UI Tree retrieval
//!
//! Retrieves the UI tree from the focused window.

fn main() {
    println!("UI Tree Example");
    println!("===============");

    // Get UI tree from focused window
    println!("Getting UI tree from focused window...");
    match usehid::get_ui_tree(Some(2), None) {
        Ok(tree) => {
            println!("Root element: role={}, title={:?}", tree.role, tree.title);
            print_tree(&tree, 0);
        }
        Err(e) => {
            eprintln!("Failed to get UI tree: {}", e);
            eprintln!("Make sure Accessibility permissions are granted.");
        }
    }

    // Find specific elements
    println!("\nSearching for buttons...");
    match usehid::find_ui_element(Some("AXButton"), None) {
        Ok(elements) => {
            println!("Found {} button(s):", elements.len());
            for elem in &elements {
                println!("  - {:?} at {:?}", elem.title, elem.bounds);
            }
        }
        Err(e) => {
            eprintln!("Find failed: {}", e);
        }
    }

    // Via Agent API
    println!("\nGetting UI tree via Agent API...");
    let mut agent = usehid::AgentHID::new();
    let result = agent.execute_json(r#"{"action": "get_ui_tree", "depth": 1}"#);
    if result.success {
        if let Some(tree) = &result.tree {
            println!("Agent got tree: role={}, title={:?}", tree.role, tree.title);
        }
    } else {
        eprintln!("Agent get_ui_tree failed: {:?}", result.error);
    }

    println!("\nDone!");
}

fn print_tree(element: &usehid::UIElement, indent: usize) {
    let prefix = "  ".repeat(indent);
    let title = element.title.as_deref().unwrap_or("");
    println!("{}[{}] {}", prefix, element.role, title);
    for child in &element.children {
        print_tree(child, indent + 1);
    }
}
