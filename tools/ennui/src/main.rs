mod audit;
mod cargo;
mod edit;
mod export;
mod fresh;
mod pick;
mod run;
mod sync;
mod web;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "ennui",
    about = "Run, edit, check and sync ennui repositories, make new apps, and audit code"
)]
struct Line {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Run a package of this repository with the play profile")]
    Run {
        #[arg(
            long,
            default_value = "on",
            help = "on links the engine as one shared library"
        )]
        dynamic: String,
        #[arg(long, help = "Record Chrome tracing spans")]
        trace: bool,
        package: String,
        #[arg(last = true, help = "Arguments for the program, after --")]
        rest: Vec<String>,
    },
    #[command(about = "Build an app and the editor, then open the editor on the app's project")]
    Edit {
        #[arg(long, default_value = "on")]
        dynamic: String,
        #[arg(long, help = "The editor clone")]
        editor: PathBuf,
        #[arg(long, default_value = "project", help = "The project folder")]
        project: PathBuf,
        package: String,
        scene: String,
    },
    #[command(about = "Clippy on the workspace, then a play build of the named packages")]
    Check {
        #[arg(long, default_value = "on")]
        dynamic: String,
        packages: Vec<String>,
    },
    #[command(about = "Check the format of this repository's crates, then run clippy")]
    Lint,
    #[command(about = "Format this repository's crates")]
    Fmt,
    #[command(about = "Copy the workspace settings, lock and toolchain into this repository")]
    Sync { engine: PathBuf },
    #[command(about = "Make a new app repository from the template")]
    New {
        #[arg(long, help = "The folder a relative target starts at")]
        from: PathBuf,
        folder: PathBuf,
    },
    #[command(
        about = "Build an app with the export profile and put it beside a copy of its project folder in target/shipped, ready to ship"
    )]
    Export {
        #[arg(long, default_value = "project", help = "The project folder")]
        project: PathBuf,
        package: String,
    },
    #[command(
        about = "Build apps for the browser into target/site: the first app is the main page, with a button to each other app in its own folder"
    )]
    Web {
        #[arg(long, help = "Serve target/site on this port after the build")]
        serve: Option<u16>,
        #[arg(required = true)]
        packages: Vec<String>,
    },
    #[command(about = "Pick an app from a list and run it")]
    Pick {
        #[arg(long, default_value = "on")]
        dynamic: String,
        #[arg(long, default_value = "apps")]
        folder: PathBuf,
        query: Vec<String>,
    },
    #[command(subcommand, about = "Audit ennui code against the data-oriented rules")]
    Audit(audit::Audit),
}

fn main() -> ExitCode {
    let line = Line::parse();
    let outcome = match line.command {
        Command::Run {
            dynamic,
            trace,
            package,
            rest,
        } => run::run(dynamic == "on", trace, &package, &rest),
        Command::Edit {
            dynamic,
            editor,
            project,
            package,
            scene,
        } => edit::edit(dynamic == "on", &editor, &project, &package, &scene),
        Command::Check { dynamic, packages } => run::check(dynamic == "on", &packages),
        Command::Lint => run::lint(true),
        Command::Fmt => run::lint(false),
        Command::Sync { engine } => sync::sync(std::path::Path::new("."), &engine),
        Command::New { from, folder } => fresh::fresh(&from, &folder),
        Command::Export { project, package } => export::export(&package, &project),
        Command::Web { serve, packages } => web::web(&packages, serve),
        Command::Pick {
            dynamic,
            folder,
            query,
        } => pick::pick(dynamic == "on", &folder, &query),
        Command::Audit(what) => return audit::run(what),
    };
    match outcome {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        Err(problem) => {
            eprintln!("ennui: {problem}");
            ExitCode::FAILURE
        }
    }
}
