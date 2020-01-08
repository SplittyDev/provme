# provme
> A Painless Hosting Provisioner.

## mkwebuser

The `mkwebuser` tool creates a user, together with a mounted volume of specified size.
The user is put into a chroot jail and an sftp account is created.
This results in a safe unescapable user environment with sftp access and quota limit.

### Process
1. Creates user `<username>` with home directory at `<user_base>/<username>`
2. Creates an ext4 volume with `<quota>` MiB at `<user_base>/<username>/volume`
3. Mounts the volume at `<mount_base>/<username>`
4. Chroot jails user `<username>` to `<mount_base>/<username>`
5. Creates an openssh sftp `nologin` entry for `<username>`

### Help Information
```
USAGE:
    mkwebuser [OPTIONS] --username <username>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --base <base>
    -q, --quota <quota>
    -u, --username <username>
    -m, --mountbase <mountbase>
```