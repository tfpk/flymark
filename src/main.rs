mod tmux;
mod choices;
mod ui;

use std::io::Write;
use anyhow::{Result, Context};
use choices::Choices;
use clap::Parser;
use tempfile::TempDir;
use tokio::{fs::File, io::AsyncReadExt};
use ui::UiParams;

#[derive(Parser, Debug)]
#[clap(version)]
pub struct Args {
    /// The endpoint you will use for marking (overrides the course + session args)
    #[clap(short, long)]
    endpoint: Option<String>,

    /// The path to the marking scheme you will use
    scheme: String,

    /// Course
    course: String,

    /// Session
    session: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let endpoint = get_endpoint(&args);
    let choices  = get_choices(&args.scheme).await
        .with_context(|| format!("Failed to read scheme file: {}", args.scheme))?;

    let tmux_session = tmux::get_tmux_session().await
        .with_context(|| format!("Failed to locate current tmux session"))?;
    
    let work_dir = move_to_work_dir()
        .with_context(|| format!("Failed to create temporary work directory"))?;

    let launch_params = UiParams::new(&args, &endpoint, &choices, &tmux_session, &work_dir);
    ui::launch_ui(launch_params).await?;

    // let auth = authenticate()
    //     .with_context(|| format!("Failed to read from stdin to authenticate"))?;

    Ok(())
}

fn get_endpoint(args: &Args) -> String {
    args.endpoint
        .as_ref()
        .cloned()
        .unwrap_or_else(|| {
            let course  = args.course.as_str();
            let session = args.session.as_str();

            format!("https://cgi.cse.unsw.edu.au/~{course}/{session}/imark/server.cgi/")
        })
}

async fn get_choices(scheme: &str) -> Result<Choices> {
    let mut file = File::open(scheme).await?;

    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    let choices = choices::parse_choices(&contents)?;

    Ok(choices)
}

fn move_to_work_dir() -> Result<TempDir> {
    let work_dir = tempfile::tempdir()?;
    std::env::set_current_dir(&work_dir)?;
    
    Ok(work_dir)
}

struct Auth {
    username: String,
    password: String,
}

fn authenticate() -> Result<Auth> {
    print!("Enter your zID: ");
    std::io::stdout().flush()?;
    
    let mut username = String::new();
    std::io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();

    let password = rpassword::prompt_password("Enter your zPass: ")?;

    Ok(Auth { username, password })
}
