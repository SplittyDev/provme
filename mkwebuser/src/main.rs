use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "mkwebuser")]
struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    base: Option<PathBuf>,

    #[structopt(short, long)]
    username: String,

    #[structopt(short, long)]
    quota: Option<f64>,
}

#[derive(Debug)]
enum AppError {
    UserCreationFailed { reason: &'static str },
}

fn main() -> Result<(), AppError> {
    // Parse arguments
    let opt: Opt = Opt::from_args();

    let username = &opt.username;
    let default_home_directory = || format!("/home/{user}", user = username);
    let home_directory = opt
        .base
        .map(|p| {
            p.to_str()
                .map(|s| s.to_string())
                .unwrap_or_else(default_home_directory)
        })
        .unwrap_or_else(default_home_directory);
    create_user(&opt.username, &home_directory)?;

    Ok(())
}

fn create_user(username: &str, home_directory: &str) -> Result<(), AppError> {
    let mut cmd = Command::new("useradd");
    cmd.args(&["--home", home_directory]);
    cmd.args(&["--comment", &format!("mkwebuser {user}", user = username)]);
    cmd.args(&["--inactive", "-1"]); // never mark user as inactive
    cmd.args(&["--shell", "/usr/sbin/nologin"]); // no interactive shell
    cmd.arg("--create-home"); // create home directory
    cmd.arg(username);
    let status: ExitStatus = cmd.status().map_err(|e| AppError::UserCreationFailed {
        reason: "Unable to get exit status",
    })?;
    if status.success() {
        Ok(())
    } else {
        // https://linux.die.net/man/8/useradd
        Err(match status.code() {
            Some(1) => AppError::UserCreationFailed {reason: "Unable to update password file"},
            Some(2) => AppError::UserCreationFailed {reason: "Invalid command syntax"},
            Some(3) => AppError::UserCreationFailed {reason: "Invalid argument to option"},
            Some(4) => AppError::UserCreationFailed {reason: "UID already in use"},
            Some(6) => AppError::UserCreationFailed {reason: "The specified group does not exist"},
            Some(9) => AppError::UserCreationFailed {reason: "Username already in use"},
            Some(10) => AppError::UserCreationFailed {reason: "Failed to update group file"},
            Some(12) => AppError::UserCreationFailed {reason: "Failed to create home directory"},
            Some(13) => AppError::UserCreationFailed {reason: "Failed to create mail spool"},
            Some(14) => AppError::UserCreationFailed {reason: "Failed to update SELinux user mapping"},
            None => AppError::UserCreationFailed {reason: "Process terminated by signal"},
            _ => AppError::UserCreationFailed {reason: "Unknown"}
        })
    }
}
