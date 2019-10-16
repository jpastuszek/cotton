use cotton::prelude::*;

#[derive(Debug, StructOpt)]
enum ScriptAction {
    /// Run `cargo check`
    Check,
    /// Build and stage for fast execution
    Build,
    /// Build, stage for fast execution and execute
    Run {
        /// Path to script file
        script: PathBuf,
        /// Arguments for the script
        arguments: Vec<String>, //TODO: OsString not supported
    },
    /// Build in debug mode and execute
    Debug,
}

/// Single file rust scritps.
#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(flatten)]
    logging: LoggingOpt,

    #[structopt(subcommand)]
    script_action: Option<ScriptAction>,
}

struct Cargo {
    script: PathBuf,
    project: PathBuf,
}

impl Cargo {
    fn new(script: PathBuf) -> Result<Cargo> {
        let script = script.canonicalize().problem_while_with(|| format!("accessing script file path {:?}", script.display()))?;
        info!("Script path: {}", script.display());

        if !script.is_file() {
            return Err(Problem::from_error(format!("Script {:?} is not a file", script.display())))
        }

        let parent_path = script
            .parent()
            .map(|p| p.to_str().ok_or_problem("Script parent path is not UTF-8 compatible"))
            .transpose()?
            .unwrap_or("./");

        let parent_path_digest = hex_digest(Some(parent_path))[0..16].to_string();
        debug!("Parent path: {} (digest: {})", parent_path, parent_path_digest);

        let project_name = script.file_stem().unwrap().to_str().ok_or_problem("Script stem is not UTF-8 compatible")?;
        debug!("Project name: {}", project_name);

        let project = app_cache(format!("project-{}-{}", parent_path_digest, project_name).as_str())?;
        info!("Project path: {}", project.display());

        let src = project.join("src");

        if !src.exists() {
            info!("Initializing cargo project");
            cmd!("cargo", "init", "--quiet", "--vcs", "none", "--name", project_name, "--bin", "--edition", "2018", &project).silent().problem_while("running cargo init")?;
        }

        Ok(Cargo {
            script,
            project,
        })
    }
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    init_logger(&args.logging, vec![module_path!()]);

    match args.script_action {
        Some(ScriptAction::Run { script, arguments }) => {
            let cargo = Cargo::new(script).or_failed_to("initialize cargo project");

        }
        _ => unimplemented!()
    }
    Ok(())
}
