use cotton::prelude::*;
use std::os::unix::fs::PermissionsExt;

const MODE_USER_EXEC: u32 = 0o100;

#[derive(Debug, StructOpt)]
enum ScriptAction {
    /// Create new scipt from template
    New {
        /// Path to script file
        script: PathBuf,
    },
    /// Run `cargo check`
    Check {
        /// Path to script file
        script: PathBuf,
    },
    /// Build and stage for fast execution
    Build {
        /// Path to script file
        script: PathBuf,
    },
    /// Build, stage for fast execution and execute
    Exec {
        /// Path to script file
        script: PathBuf,

        /// Arguments for the script
        arguments: Vec<String>, //TODO: OsString not supported
    },
    /// Build and run tests
    Test {
        /// Path to script file
        script: PathBuf,
    },
    /// Remove all cached build files related to scipt file
    Clean {
        /// Path to script file
        script: PathBuf,
    },
    /// Remove all cached build files
    CleanAll,
}

/// Single file rust scritps.
#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(flatten)]
    logging: LoggingOpt,

    #[structopt(subcommand)]
    script_action: ScriptAction,
}

#[derive(Debug, Clone, Copy)]
enum CargoMode {
    Silent,
    Verbose,
}

#[derive(Debug, Clone, Copy)]
enum CargoState {
    ScriptDiffers,
    NoBinary,
    BinaryOutdated,
    UpToDate,
}

impl CargoState {
    fn needs_update(&self) -> bool {
        match self {
            CargoState::ScriptDiffers => true,
            _ => false
        }
    }

    fn needs_build(&self) -> bool {
        match self {
            CargoState::ScriptDiffers => true,
            CargoState::NoBinary | CargoState::BinaryOutdated => true,
            _ => false
        }
    }
}

#[derive(Debug)]
struct Cargo {
    project_name: String,
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

        let project_name = script.file_stem().unwrap().to_str().ok_or_problem("Script stem is not UTF-8 compatible")?.to_owned();
        debug!("Project name: {}", project_name);

        let project = app_cache(format!("project-{}-{}", parent_path_digest, project_name).as_str())?;
        debug!("Project path: {}", project.display());

        if !project.join("src").exists() {
            info!("Initializing cargo project in {}", project.display());
            cmd!("cargo", "init", "--quiet", "--vcs", "none", "--name", &project_name, "--bin", "--edition", "2018", &project).silent().problem_while("running cargo init")?;
        }

        Ok(Cargo {
            project_name,
            script,
            project,
        })
    }

    fn release_target_path(&self) -> PathBuf {
        self.project.join("target").join("release").join(&self.project_name)
    }

    fn main_path(&self) -> PathBuf {
        self.project.join("src").join("main.rs")
    }

    fn manifest_path(&self) -> PathBuf {
        self.project.join("Cargo.toml")
    }

    fn binary_path(&self) -> PathBuf {
        self.project.join(&self.project_name)
    }

    fn script_content(&self) -> Result<String> {
        // TODO: read up to _DATA_ marker and provide File object seeked at first byte after it
        Ok(fs::read_to_string(&self.script).problem_while("reading script contents")?)
    }

    fn script_template<'i>(name: &str) -> String {
        format!(include_str!("../template.rs"), name = name)
    }

    fn manifest_content(&self) -> Result<String> {
        let manifest = self.script_content()?
            .lines()
            .map(|l| l.trim())
            .skip_while(|l| *l != "/* Cargo.toml")
            .skip(1)
            .take_while(|l| *l != "*/")
            .join("\n");

        if manifest.is_empty() {
            Err(Problem::from_error("Cargo.toml manifest not found in the script"))
        } else {
            Ok(manifest)
        }
    }

    /// Checks state of the repository and script.
    fn state(&self) -> Result<CargoState> {
        if hex_digest(Some(self.script_content()?.as_str())) != hex_digest_file(&self.main_path())? {
            return Ok(CargoState::ScriptDiffers)
        }

        let binary_path = self.binary_path();

        if !binary_path.is_file() {
            return Ok(CargoState::NoBinary)
        }

        // binary should be newer than the script file or we have a failed build of the script
        if fs::metadata(&binary_path)?.modified()? < fs::metadata(&self.script)?.modified()? {
            return Ok(CargoState::BinaryOutdated)
        }

        Ok(CargoState::UpToDate)
    }

    /// Updates repository from the script file.
    fn update(&self) -> Result<()> {
        info!("Updating project");

        fs::write(&self.main_path(), self.script_content()?).problem_while("writing new main.rs file")?;
        fs::write(&self.manifest_path(), self.manifest_content()?).problem_while("writing new Cargo.toml file")?;

        Ok(())
    }

    /// Builds cargo project.
    fn build(&self, mode: CargoMode) -> Result<()> {
        info!("Building release target");
        match mode {
            CargoMode::Silent => cmd!("cargo", "build", "--release").dir(&self.project).silent(),
            CargoMode::Verbose => cmd!("cargo", "build", "--color", "always", "--release").dir(&self.project).exec(),
        }
        .problem_while("running cargo build")?;

        fs::rename(self.release_target_path(), self.binary_path()).problem_while("moving compiled target final location")?;

        Ok(())
    }

    /// Returns true if execute has binary to run.
    fn binary_built(&self) -> bool {
        self.binary_path().is_file()
    }

    /// Replace this image with imange of the binary.
    fn execute<I>(&self, args: I) -> Result<()> where I: IntoIterator, I::Item: AsRef<OsStr> {
        // TODO: replace return with ! when stable
        Err(Problem::from_error(exec(self.binary_path(), args)).problem_while("executing compiled binary"))
    }

    /// Prepares executable
    fn ensure_updated(&self) -> Result<()> {
        let state = self.state()?;
        if state.needs_update() {
            self.update()?;
        }
        Ok(())
    }

    /// Prepares executable
    fn ensure_built(&self, mode: CargoMode) -> Result<()> {
        let state = self.state()?;
        debug!("State: {:?}", state);
        if state.needs_update() {
            self.update()?;
        }
        if state.needs_build() {
            self.build(mode)?;
        }
        Ok(())
    }

    /// Runs 'cargo check' on updated repository
    fn check(&self) -> Result<()> {
        self.update()?;
        cmd!("cargo", "check", "--color", "always").dir(&self.project).exec().problem_while("running cargo check")?;
        Ok(())
    }

    /// Runs 'cargo test' on updated repository
    fn test(&self) -> Result<()> {
        self.update()?;
        cmd!("cargo", "test", "--color", "always").dir(&self.project).exec().problem_while("running cargo test")?;
        Ok(())
    }

    fn clean(&self) -> Result<()> {
        info!("Removing content of {}", self.project.display());
        fs::remove_dir_all(&self.project)?;
        Ok(())
    }

    fn clean_all() -> Result<()> {
        let project_root = app_cache(None)?;

        info!("Removing content of {}", project_root.display());
        fs::remove_dir_all(&project_root)?;
        Ok(())
    }
}

fn main() -> Result<()> {
    if let Some(script) = std::env::args().skip(1).next().and_then(|arg1| arg1.ends_with(".rs").as_some(arg1)) {
        ::problem::format_panic_to_stderr();

        let cargo = Cargo::new(PathBuf::from(script)).or_failed_to("initialize cargo project");

        if !cargo.binary_built() {
            cargo.ensure_built(CargoMode::Silent).or_failed_to("build script");
        }

        cargo.execute(std::env::args().skip(2)).unwrap();

        unreachable!()
    }

    let args = Cli::from_args();
    init_logger(&args.logging, vec![module_path!()]);

    match args.script_action {
        ScriptAction::New { script } => {
            let project_name = script.file_stem().unwrap().to_str().ok_or_problem("Script stem is not UTF-8 compatible")?.to_owned();
            info!("Generating new sciprt {:?} in {}", project_name, script.display());

            fs::write(&script, &Cargo::script_template(&project_name)).or_failed_to("write template to new scipt file");

            let file = File::open(&script).unwrap();
            let meta = file.metadata().unwrap();
            let mut perm = meta.permissions();
            perm.set_mode(perm.mode() | MODE_USER_EXEC);
            drop(file);

            fs::set_permissions(&script, perm).or_failed_to("to set permission");
        }
        ScriptAction::Exec { script, arguments } => {
            let cargo = Cargo::new(script).or_failed_to("initialize cargo project");
            cargo.ensure_built(CargoMode::Verbose).or_failed_to("update_and_build script binary");
            cargo.execute(arguments).unwrap();
        }
        ScriptAction::Build { script } => {
            let cargo = Cargo::new(script).or_failed_to("initialize cargo project");
            cargo.ensure_built(CargoMode::Verbose).or_failed_to("build script binary");
        }
        ScriptAction::Check { script } => {
            let cargo = Cargo::new(script).or_failed_to("initialize cargo project");
            cargo.ensure_updated().or_failed_to("update cargo project");
            cargo.check().or_failed_to("check script");
        }
        ScriptAction::Test { script } => {
            let cargo = Cargo::new(script).or_failed_to("initialize cargo project");
            cargo.ensure_updated().or_failed_to("update cargo project");
            cargo.test().or_failed_to("test script");
        }
        ScriptAction::Clean { script } => {
            let cargo = Cargo::new(script).or_failed_to("initialize cargo project");
            cargo.clean().or_failed_to("clean script repository");
        }
        ScriptAction::CleanAll => {
            Cargo::clean_all().or_failed_to("clean script repository");
        }
    }
    Ok(())
}
