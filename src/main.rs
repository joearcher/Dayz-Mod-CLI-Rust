use std::fs;
use std::path::Path;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use inquire::{MultiSelect, Text};

/// A CLI tool for creating DayZ mod skeletons.
#[derive(Parser)]
#[command(name = "dayzmod", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generates the base folder and file structure for a new DayZ mod.
    Create,
}

/// The config.cpp template, baked into the binary at compile time.
const STUB: &str = include_str!("config.cpp.stub");

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create => create(),
    }
}

fn create() -> ExitCode {
    // Ask for the mod name. `inquire` enforces a non-empty answer by default.
    let name = match Text::new("What is the new mod called?")
        .with_placeholder("My_test_mod")
        .with_help_message("This will be used for the base class names and folder paths.")
        .prompt()
    {
        Ok(value) => value,
        // The user cancelled (Esc / Ctrl-C).
        Err(_) => return ExitCode::from(1),
    };

    let mod_name = sanitize(&name);

    if mod_name.is_empty() {
        eprintln!("Please provide a valid mod name.");
        return ExitCode::from(1);
    }

    if Path::new(&mod_name).exists() {
        eprintln!(
            "A folder with this mod name already exists in the current path, please choose a different name."
        );
        return ExitCode::from(1);
    }

    let script_modules = match MultiSelect::new(
        &format!("Which script modules should be enabled for {mod_name}?"),
        vec!["3_Game", "4_World", "5_Mission"],
    )
    .prompt()
    {
        Ok(selection) => selection,
        Err(_) => return ExitCode::from(1),
    };

    if let Err(err) = build_mod(&mod_name, &script_modules) {
        eprintln!("Failed to create mod: {err}");
        return ExitCode::from(1);
    }

    println!("Mod folder structure created successfully.");
    ExitCode::SUCCESS
}

/// Mirror the PHP sanitisation: spaces become underscores, quote characters are stripped.
fn sanitize(name: &str) -> String {
    name.replace(' ', "_")
        .replace('\'', "")
        .replace('"', "")
        .replace('`', "")
}

/// Create the folder structure and write config.cpp.
fn build_mod(mod_name: &str, script_modules: &[&str]) -> std::io::Result<()> {
    fs::create_dir_all(mod_name)?;
    fs::create_dir_all(format!("{mod_name}/Scripts"))?;

    let mut stub = STUB.to_string();

    stub = handle_module(
        stub,
        script_modules,
        "3_Game",
        "{ gamescript module }",
        "{ game dep }",
        game_module_string(),
        "\"Game\",",
        &format!("{mod_name}/Scripts/3_Game"),
    )?;

    stub = handle_module(
        stub,
        script_modules,
        "4_World",
        "{ worldscript module }",
        "{ world dep }",
        world_module_string(),
        "\"World\",",
        &format!("{mod_name}/Scripts/4_World"),
    )?;

    stub = handle_module(
        stub,
        script_modules,
        "5_Mission",
        "{ missionscript module }",
        "{ mission dep }",
        mission_module_string(),
        "\"Mission\",",
        &format!("{mod_name}/Scripts/5_Mission"),
    )?;

    let output = stub.replace("{ mod_name }", mod_name);

    fs::write(format!("{mod_name}/config.cpp"), output)?;

    Ok(())
}

/// Inject (or clear) a script module's placeholders and create its folder when enabled.
#[allow(clippy::too_many_arguments)]
fn handle_module(
    stub: String,
    script_modules: &[&str],
    key: &str,
    module_placeholder: &str,
    dep_placeholder: &str,
    module_string: &str,
    dep_string: &str,
    module_dir: &str,
) -> std::io::Result<String> {
    if !script_modules.contains(&key) {
        return Ok(stub
            .replace(module_placeholder, "")
            .replace(dep_placeholder, ""));
    }

    let stub = stub
        .replace(module_placeholder, module_string)
        .replace(dep_placeholder, dep_string);

    fs::create_dir_all(module_dir)?;

    Ok(stub)
}

fn game_module_string() -> &'static str {
    "class gameScriptModule
            {
                value=\"\";
                files[]=
                    {
                        \"{ mod_name }/Scripts/3_Game\"
                    };
            };"
}

fn world_module_string() -> &'static str {
    "class worldScriptModule
            {
                value=\"\";
                files[]=
                    {
                        \"{ mod_name }/Scripts/4_World\"
                    };
            };"
}

fn mission_module_string() -> &'static str {
    "class missionScriptModule
            {
                value=\"\";
                files[]=
                    {
                        \"{ mod_name }/Scripts/5_Mission\"
                    };
            };"
}
