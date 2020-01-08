use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "mkwebuser")]
struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    base: Option<PathBuf>,

    #[structopt(short, long)]
    username: String,

    #[structopt(short, long)]
    quota: Option<u64>,
}

#[derive(Debug)]
enum AppError {
    UserCreationFailed { reason: &'static str },
    UserSpaceCreationFailed { reason: &'static str },
    UserSpaceFormattingFailed { reason: &'static str },
}

#[derive(Debug)]
struct User {
    username: String,
    base_directory: String,
    home_directory: String,
}

#[derive(Debug)]
struct UserSpace {
    name: String,
    path: String,
    size_mb: u64,
}

#[derive(Debug)]
struct WebSpaceAccount {
    user: User,
    userspace: UserSpace,
}

fn main() -> Result<(), AppError> {
    // Parse arguments
    let opt: Opt = Opt::from_args();

    // Create web space account
    let acc = {
        // Create user
        let user = create_user(&opt)?;

        // Create user space with quota
        let userspace = create_user_space(&opt, &user)?;

        // Instantiate data structure
        WebSpaceAccount { user, userspace }
    };

    println!(
        "[SUCCESS] User {{ name: {user} }}; Userspace {{ name: {space}; size: {size} }}",
        user = acc.user.username,
        space = acc.userspace.name,
        size = acc.userspace.size_mb,
    );

    Ok(())
}

fn create_user(opt: &Opt) -> Result<User, AppError> {
    // Prepare arguments
    let username = &opt.username;
    let default_home_directory = || "/home".to_string();
    let base_directory = opt
        .base
        .clone()
        .map(|p| {
            p.to_str()
                .map(|s| s.to_string())
                .unwrap_or_else(default_home_directory)
        })
        .unwrap_or_else(default_home_directory);

    // Create user
    invoke_create_user(&opt.username, &base_directory)?;

    // Log
    println!(
        "User created: {user} ({base_dir}/{user})",
        user = username,
        base_dir = base_directory
    );

    // Instantiate data structure
    Ok(User {
        username: username.to_string(),
        home_directory: format!("{}/{}", base_directory, username),
        base_directory: base_directory,
    })
}

fn invoke_create_user(username: &str, home_directory: &str) -> Result<(), AppError> {
    let mut cmd = Command::new("useradd");
    cmd.args(&["--base-dir", home_directory]);
    cmd.args(&["--comment", &format!("mkwebuser {user}", user = username)]);
    cmd.args(&["--inactive", "-1"]); // never mark user as inactive
    cmd.args(&["--shell", "/usr/sbin/nologin"]); // no interactive shell
    cmd.arg("--create-home"); // create home directory
    cmd.arg(username);
    let status: ExitStatus = cmd.status().map_err(|_| AppError::UserCreationFailed {
        reason: "Unable to get exit status",
    })?;
    if status.success() {
        Ok(())
    } else {
        // https://linux.die.net/man/8/useradd
        Err(match status.code() {
            Some(1) => AppError::UserCreationFailed {
                reason: "Unable to update password file",
            },
            Some(2) => AppError::UserCreationFailed {
                reason: "Invalid command syntax",
            },
            Some(3) => AppError::UserCreationFailed {
                reason: "Invalid argument to option",
            },
            Some(4) => AppError::UserCreationFailed {
                reason: "UID already in use",
            },
            Some(6) => AppError::UserCreationFailed {
                reason: "The specified group does not exist",
            },
            Some(9) => AppError::UserCreationFailed {
                reason: "Username already in use",
            },
            Some(10) => AppError::UserCreationFailed {
                reason: "Failed to update group file",
            },
            Some(12) => AppError::UserCreationFailed {
                reason: "Failed to create home directory",
            },
            Some(13) => AppError::UserCreationFailed {
                reason: "Failed to create mail spool",
            },
            Some(14) => AppError::UserCreationFailed {
                reason: "Failed to update SELinux user mapping",
            },
            None => AppError::UserCreationFailed {
                reason: "Process terminated by signal",
            },
            _ => AppError::UserCreationFailed { reason: "Unknown" },
        })
    }
}

fn create_user_space(opt: &Opt, user: &User) -> Result<UserSpace, AppError> {
    // Prepare arguments
    let name = "volume";
    let path = format!(
        "{home_dir}/{name}",
        home_dir = user.home_directory,
        name = name
    );
    let quota = opt.quota.unwrap_or(1024_u64);
    // Create user space
    invoke_create_user_space(&path, quota)?;

    // Log
    println!("Space created: {size}M ({path})", size = quota, path = path,);

    // Format user space
    invoke_format_user_space(&path)?;

    // Log
    println!("Space formatted: ext4 ({path})", path = path,);

    // Instantiate data structure
    Ok(UserSpace {
        name: name.to_string(),
        path: path,
        size_mb: quota,
    })
}

fn invoke_create_user_space<P>(path: &P, quota_mb: u64) -> Result<(), AppError>
where
    P: AsRef<str>,
{
    let path: &str = path.as_ref();
    let mut cmd = Command::new("dd");
    cmd.arg(format!("if=/dev/zero"));
    cmd.arg(format!("of={path}", path = path));
    cmd.arg(format!("bs={size}M", size = quota_mb));
    cmd.arg("count=1");
    cmd.stderr(Stdio::null());
    cmd.stdout(Stdio::null());
    let status: ExitStatus = cmd
        .status()
        .map_err(|_| AppError::UserSpaceCreationFailed {
            reason: "Unable to get exit status",
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::UserSpaceCreationFailed { reason: "dd error" })
    }
}

fn invoke_format_user_space<P>(path: &P) -> Result<(), AppError>
where
    P: AsRef<str>,
{
    let path: &str = path.as_ref();
    let mut cmd = Command::new("mkfs.ext4");
    cmd.arg(path);
    cmd.stderr(Stdio::null());
    cmd.stdout(Stdio::null());
    let status: ExitStatus = cmd
        .status()
        .map_err(|_| AppError::UserSpaceFormattingFailed {
            reason: "Unable to get exit status",
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::UserSpaceFormattingFailed {
            reason: "mkfs.ext4 error",
        })
    }
}
