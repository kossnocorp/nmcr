use crate::prelude::*;
use nmcr_md_parser::prelude::*;
use nmcr_mcp::template::render_template;
use nmcr_types::{Template, TemplateArgType};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::io::{self, Write};

#[derive(Args)]
pub struct GenArgs {
    /// Template to generate from (e.g. "react_component", "rust_package_cargo_toml")
    #[arg(short, long)]
    pub template: Option<String>,
    
    /// Output path where the generated content should be written
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    
    /// List available templates
    #[arg(short, long)]
    pub list: bool,
}

#[derive(Args)]
pub struct GenCmd {}

impl GenCmd {
    pub async fn run(args: &CliCommandProject<GenArgs>) -> Result<()> {
        let project = args.load_project()?;
        let templates = project.template_paths()?;
        
        // Collect all available templates
        let mut available_templates = Vec::new();
        
        for template_path in templates {
            match parse_file(&template_path) {
                Ok(ParsedMarkdown::Template(template)) => {
                    available_templates.push(template);
                }
                Ok(ParsedMarkdown::Collection(collection)) => {
                    available_templates.extend(collection.templates);
                }
                Err(err) => {
                    eprintln!("Failed to parse template at {}: {}", template_path.display(), err);
                }
            }
        }
        
        if available_templates.is_empty() {
            eprintln!("No templates found in project.");
            return Ok(());
        }
        
        // List templates if requested
        if args.local.list {
            Self::list_templates(&available_templates);
            return Ok(());
        }
        
        // Get template name from args or prompt user
        let template_name = match &args.local.template {
            Some(name) => name.clone(),
            None => Self::prompt_for_template(&available_templates)?,
        };
        
        // Find the selected template
        let selected_template = available_templates
            .iter()
            .find(|t| Self::sanitize_tool_name(&t.name) == template_name)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_name))?;
        
        // Collect template arguments
        let template_args = Self::collect_template_args(selected_template)?;
        
        // Generate content
        let rendered_content = render_template(&selected_template.content, &template_args);
        
        // Write to file or stdout
        match &args.local.output {
            Some(output_path) => {
                Self::write_to_file(&rendered_content, output_path)?;
                println!("Generated content written to: {}", output_path.display());
            }
            None => {
                println!("Generated content:");
                println!("---");
                println!("{}", rendered_content);
            }
        }
        
        Ok(())
    }
    
    fn list_templates(templates: &[Template]) {
        println!("Available templates:");
        for template in templates {
            let tool_name = Self::sanitize_tool_name(&template.name);
            let description = if template.description.trim().is_empty() {
                "(no description)"
            } else {
                &template.description
            };
            
            println!("  {} - {}", tool_name, description);
            
            if !template.args.items.is_empty() {
                println!("    Arguments:");
                for arg in &template.args.items {
                    let desc = if arg.description.trim().is_empty() {
                        "(no description)"
                    } else {
                        &arg.description
                    };
                    println!("      {} - {}", arg.name, desc);
                }
            }
        }
    }
    
    fn prompt_for_template(templates: &[Template]) -> Result<String> {
        println!("Select a template:");
        for (i, template) in templates.iter().enumerate() {
            let tool_name = Self::sanitize_tool_name(&template.name);
            println!("  {}: {}", i + 1, tool_name);
        }
        
        loop {
            print!("Enter template number (1-{}): ", templates.len());
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if let Ok(num) = input.trim().parse::<usize>() {
                if num >= 1 && num <= templates.len() {
                    return Ok(Self::sanitize_tool_name(&templates[num - 1].name));
                }
            }
            
            println!("Invalid selection. Please enter a number between 1 and {}.", templates.len());
        }
    }
    
    fn collect_template_args(template: &Template) -> Result<JsonMap<String, JsonValue>> {
        let mut args = JsonMap::new();
        
        if template.args.items.is_empty() {
            return Ok(args);
        }
        
        println!("Template '{}' requires arguments:", template.name);
        
        for arg in &template.args.items {
            print!("{} ({}): ", arg.name, arg.description);
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let value = input.trim().to_string();
            
            if !value.is_empty() {
                // Try to parse as different types based on the argument type
                let json_value = match arg.kind {
                    TemplateArgType::Boolean => {
                        match value.to_lowercase().as_str() {
                            "true" | "yes" | "y" | "1" => JsonValue::Bool(true),
                            "false" | "no" | "n" | "0" => JsonValue::Bool(false),
                            _ => JsonValue::String(value),
                        }
                    }
                    TemplateArgType::Number => {
                        value.parse::<f64>()
                            .map(|n| JsonValue::Number(serde_json::Number::from_f64(n).unwrap()))
                            .unwrap_or_else(|_| JsonValue::String(value))
                    }
                    _ => JsonValue::String(value),
                };
                
                args.insert(arg.name.clone(), json_value);
            }
        }
        
        Ok(args)
    }
    
    fn write_to_file(content: &str, path: &PathBuf) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        fs::write(path, content)?;
        Ok(())
    }
    
    fn sanitize_tool_name(name: &str) -> String {
        let mut cleaned = String::new();
        let mut last_was_separator = false;
        for ch in name.chars() {
            if ch.is_ascii_alphanumeric() {
                cleaned.push(ch.to_ascii_lowercase());
                last_was_separator = false;
            } else if !cleaned.is_empty() && !last_was_separator {
                cleaned.push('_');
                last_was_separator = true;
            }
        }
        if cleaned.ends_with('_') {
            cleaned.pop();
        }
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn sanitize_tool_name_basic() {
        assert_eq!(GenCmd::sanitize_tool_name("React Component"), "react_component");
        assert_eq!(GenCmd::sanitize_tool_name("  API::Client  "), "api_client");
        assert_eq!(GenCmd::sanitize_tool_name("???"), "");
        assert_eq!(GenCmd::sanitize_tool_name("Package Cargo.toml"), "package_cargo_toml");
    }

    #[test]
    fn write_to_file_creates_directories() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_path = temp_dir.path().join("nested").join("deep").join("test.txt");
        let content = "Hello, world!";

        GenCmd::write_to_file(content, &test_path)?;

        assert!(test_path.exists());
        assert_eq!(fs::read_to_string(&test_path)?, content);

        Ok(())
    }

    #[test]
    fn write_to_file_overwrites_existing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_path = temp_dir.path().join("test.txt");
        
        // Write initial content
        GenCmd::write_to_file("first", &test_path)?;
        assert_eq!(fs::read_to_string(&test_path)?, "first");

        // Overwrite with new content
        GenCmd::write_to_file("second", &test_path)?;
        assert_eq!(fs::read_to_string(&test_path)?, "second");

        Ok(())
    }

    #[test]
    fn sanitize_tool_name_edge_cases() {
        assert_eq!(GenCmd::sanitize_tool_name(""), "");
        assert_eq!(GenCmd::sanitize_tool_name("a"), "a");
        assert_eq!(GenCmd::sanitize_tool_name("A"), "a");
        assert_eq!(GenCmd::sanitize_tool_name("1test"), "1test");
        assert_eq!(GenCmd::sanitize_tool_name("test_"), "test");
        assert_eq!(GenCmd::sanitize_tool_name("_test"), "test");
        assert_eq!(GenCmd::sanitize_tool_name("test___name"), "test_name");
    }
}
