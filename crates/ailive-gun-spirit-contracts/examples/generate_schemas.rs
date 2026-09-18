use ailive_gun_spirit_contracts::generated_schema_documents;
use std::path::PathBuf;

fn main() {
    if let Err(error) = run() {
        eprintln!("schema generation failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let check_only = std::env::args().any(|argument| argument == "--check");
    let schema_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemas");
    let mut drift = Vec::new();

    let schemas = generated_schema_documents().map_err(|error| error.to_string())?;
    for schema in schemas {
        let path = schema_root.join(schema.file_name);
        if check_only {
            match std::fs::read_to_string(&path) {
                Ok(committed) if committed == schema.json => {}
                Ok(_) => drift.push(format!("{} differs from generated output", path.display())),
                Err(error) => drift.push(format!("cannot read {}: {error}", path.display())),
            }
        } else {
            std::fs::write(&path, schema.json)
                .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
            println!("generated {}", path.display());
        }
    }

    if !drift.is_empty() {
        for message in drift {
            eprintln!("schema drift: {message}");
        }
        std::process::exit(1);
    }

    Ok(())
}
