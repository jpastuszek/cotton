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
    /// Remove all cached build files
    Clean {
        /// Path to script file
        script: PathBuf,
    },
}

/// Single file rust scritps.
#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(flatten)]
    logging: LoggingOpt,

    #[structopt(subcommand)]
    script_action: Option<ScriptAction>,
}

#[derive(Debug)]
struct Cargo {
    project_name: String,
    script: PathBuf,
    project: PathBuf,
    main: PathBuf,
    manifest: PathBuf,
    manifest_orig: PathBuf,
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

        let src = project.join("src");
        let main = src.join("main.rs");
        let manifest = project.join("Cargo.toml");
        let manifest_orig = project.join("Cargo.toml.orig");

        if !src.exists() {
            info!("Initializing cargo project in {}", project.display());
            cmd!("cargo", "init", "--quiet", "--vcs", "none", "--name", &project_name, "--bin", "--edition", "2018", &project).silent().problem_while("running cargo init")?;

            fs::remove_file(&main).or_failed_to("remove main.rs form new repository");

            if !manifest_orig.exists() {
                assert!(manifest.exists());
                debug!("Keeping copy of original manifest: {}", manifest_orig.display());
                fs::copy(&manifest, &manifest_orig).problem_while("copying manifest")?;
            }
        }

        assert!(manifest.exists(), "Bad repository state: missing Cargo.toml");
        assert!(manifest_orig.exists(), "Bad repository state: missing: Cargo.toml.orig");

        Ok(Cargo {
            project_name,
            script,
            project,
            main,
            manifest,
            manifest_orig,
        })
    }

    fn release_target(&self) -> Option<PathBuf> {
        let target = self.project.join("target").join("release").join(&self.project_name);
        if target.is_file() {
            Some(target)
        } else {
            None
        }
    }

    fn binary_path(&self) -> PathBuf {
        self.project.join(&self.project_name)
    }

    fn binary(&self) -> Option<PathBuf> {
        let binary = self.binary_path();
        if binary.is_file() {
            Some(binary)
        } else {
            None
        }
    }

    fn script_content(&self) -> Result<String> {
        // TODO: read up to _DATA_ marker and provide File object seeked at first byte after it
        Ok(fs::read_to_string(&self.script).problem_while("reading script contents")?)
    }

    fn modified(&self) -> bool {
        //TODO: just check mtime?
        hex_digest(Some(self.script_content().or_failed_to("read sript file").as_str())) != hex_digest_file(&self.main).or_failed_to("digest main.rs")
    }

    /// Builds project.
    ///
    /// Project files are updated from script source.
    /// Cargo is called to build the target file.
    /// Target file is moved into new 'active' target locatoin which is atomi so that caller
    /// can continue to call the script as it is beign built.
    fn build(&self) -> Result<()> {
        assert!(self.release_target().is_none(), "Bad repository state: target file present");
        info!("Building release target");

        cmd!("cargo", "build", "--release").dir(&self.project).silent().problem_while("running cargo build")?;

        let target = self.release_target().expect("Build failed to create release target");
        fs::rename(target, self.binary_path()).problem_while("moving compiled target final location")?;

        Ok(())
    }

    /// Checks if there are script has been updated and updates repository from the script file.
    fn update(&self) -> Result<()> {
        if self.main.exists() && !self.modified() {
            return Ok(())
        }

        info!("Updating project");
        fs::write(&self.main, self.script_content()?).problem_while("writing new main.rs file")?;

        Ok(())
    }

    /// Just runs the binary building it if not built at all
    fn run<I>(&self, args: I) -> Result<()> where I: IntoIterator, I::Item: AsRef<OsStr> {
        if let Some(binary) = self.binary() {
            // TODO: replace return with ! when stable
            Err(Problem::from_error(exec(binary, args)).problem_while("executing compiled binary"))
        } else {
            self.update()?;
            assert!(self.main.exists());
            self.build()?;
            assert!(self.binary().is_some());
            self.run(args)
        }
    }

    fn clean(&self) -> Result<()> {
        info!("Removing content of {}", self.project.display());
        fs::remove_dir_all(&self.project)?;
        Ok(())
    }
}

fn main() -> Result<()> {
    if let Some(script) = std::env::args().skip(1).next().and_then(|arg1| arg1.ends_with(".rs").as_some(arg1)) {
        ::problem::format_panic_to_stderr();
        let cargo = Cargo::new(PathBuf::from(script)).or_failed_to("initialize cargo project");
        cargo.run(std::env::args().skip(2)).or_failed_to("run script");
        unreachable!()
    }

    let args = Cli::from_args();
    init_logger(&args.logging, vec![module_path!()]);

    match args.script_action {
        Some(ScriptAction::Run { script, arguments }) => {
            let cargo = Cargo::new(script).or_failed_to("initialize cargo project");
            trace!("{:?}", cargo);
            cargo.run(arguments).or_failed_to("run script");
        }
        Some(ScriptAction::Clean { script }) => {
            let cargo = Cargo::new(script).or_failed_to("initialize cargo project");
            cargo.clean().or_failed_to("clean script repository");
        }
        _ => unimplemented!()
    }
    Ok(())
}
