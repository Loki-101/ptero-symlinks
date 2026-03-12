// ── Imports ─────────────────────────────────────────────────────────────────

use serde::Deserialize;
use serde_json::Value as Json;
use std::collections::HashSet;
use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use uzers::{get_current_uid, get_user_by_name, get_user_by_uid, get_user_groups, os::unix::UserExt};

// ── Platform Guard & Constants ──────────────────────────────────────────────

#[cfg(not(target_os = "linux"))]
compile_error!("Linux-only");

static DEFAULT_WINGS_CONFIG: &str = "/etc/pterodactyl/config.yml";

// ── Types ───────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct WingsConfig { uuid: String, system: System }
#[derive(Deserialize)]
struct System { data: String, user: SystemUser }
#[derive(Deserialize)]
struct SystemUser { gid: Option<u32> }

#[derive(Clone, Debug)]
struct Server { uuid: String, name: String }

// ── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug)]
enum Error {
    NotRoot,
    NoUser,
    NoHome(String),
    NeedAcl,
    AclDisabled,
    AclVerify(io::Error),
    NoApiKey,
    NoPanel,
    BadPanel,
    BadConfig(&'static str),
    NodeNotFound,
    Io(io::Error),
    Yaml(serde_yaml_ng::Error),
    Http(Box<ureq::Error>),
    Symlink { link: PathBuf, target: PathBuf, source: io::Error },
    SetAcl { dir: PathBuf, source: io::Error },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRoot          => write!(f, "must run as root/sudo"),
            Self::NoUser           => write!(f, "cannot resolve real user"),
            Self::NoHome(u)        => write!(f, "home dir not found for {u}"),
            Self::NeedAcl          => write!(f, "ACL tools (setfacl/getfacl) not found. Please install acl package."),
            Self::AclDisabled      => write!(f, "ACL is not enabled on the filesystem. Please mount with 'acl' option."),
            Self::AclVerify(e)     => write!(f, "Failed to verify ACL support: {e}"),
            Self::NoApiKey         => write!(f, "API key missing"),
            Self::NoPanel          => write!(f, "panel URL missing"),
            Self::BadPanel         => write!(f, "panel must start with http:// or https://"),
            Self::BadConfig(msg)   => write!(f, "{msg}"),
            Self::NodeNotFound     => write!(f, "node not found"),
            Self::Io(e)            => write!(f, "{e}"),
            Self::Yaml(e)          => write!(f, "{e}"),
            Self::Http(e)          => write!(f, "{e}"),
            Self::Symlink { link, target, source } => write!(f, "symlink {} -> {}: {}", link.display(), target.display(), source),
            Self::SetAcl { dir, source } => write!(f, "Failed to set ACL on {}: {source}", dir.display()),
        }
    }
}

impl From<io::Error> for Error { fn from(e: io::Error) -> Self { Self::Io(e) } }
impl From<serde_yaml_ng::Error> for Error { fn from(e: serde_yaml_ng::Error) -> Self { Self::Yaml(e) } }
impl From<ureq::Error> for Error { fn from(e: ureq::Error) -> Self { Self::Http(Box::new(e)) } }
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e)                  => Some(e),
            Self::Yaml(e)                => Some(e),
            Self::Http(e)                => Some(e.as_ref()),
            Self::AclVerify(e)           => Some(e),
            Self::SetAcl { source, .. }  => Some(source),
            Self::Symlink { source, .. } => Some(source),
            _                            => None,
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Checks whether a command exists on the system via `which`.
fn has_command(name: &str) -> bool {
    Command::new("which").arg(name).output()
        .map(|o| o.status.success()).unwrap_or(false)
}

fn safe(s: &str) -> String { s.chars().map(|c| if c.is_ascii_alphanumeric() || "-_. ".contains(c) { c } else { '_' }).collect() }

fn collect_dirs_and_ancestors(paths: &[PathBuf]) -> HashSet<PathBuf> {
    let mut dirs = HashSet::new();
    for path in paths {
        if path.is_dir() { dirs.insert(path.clone()); }
        let mut current = path.clone();
        while let Some(parent) = current.parent() {
            if parent == Path::new("/") { break; }
            dirs.insert(parent.to_path_buf());
            current = parent.to_path_buf();
        }
    }
    dirs
}

// ── Entry Point ─────────────────────────────────────────────────────────────

fn main() {
    if let Err(e) = run() {
        eprint!("{e}");
        let mut src = std::error::Error::source(&e);
        while let Some(cause) = src {
            eprint!(": {cause}");
            src = cause.source();
        }
        eprintln!();
        std::process::exit(1);
    }
}

// ── Core Logic ──────────────────────────────────────────────────────────────

fn run() -> Result<()> {
    let (api_key, panel, home, real_user) = env_cfg()?;
    let WingsConfig { uuid: node_uuid, system } = read_wings_config(&env::var("WINGS_CONFIG").unwrap_or_else(|_| DEFAULT_WINGS_CONFIG.into()))?;

    let node_id = fetch_node_id(&panel, &api_key, &node_uuid)?;
    let servers = fetch_servers_on_node(&panel, &api_key, node_id)?;

    let link_dir = home.join("pterodactyl");
    fs::create_dir_all(&link_dir)?;
    prune_dangling(&link_dir)?;

    let mut made = 0usize;
    for s in servers {
        if s.uuid.len() < 8 { continue; }
        let target = Path::new(&system.data).join(&s.uuid);
        if !target.exists() { continue; }
        let name = format!("{}-{}", safe(&s.name), &s.uuid[..8]);
        let link = link_dir.join(name);
        if link.exists() { continue; }
        unix_fs::symlink(&target, &link).map_err(|e| Error::Symlink { link: link.clone(), target: target.clone(), source: e })?;
        made += 1;
    }

    if let Some(gid) = system.user.gid { 
        group_management(gid, &real_user, &link_dir, Path::new(&system.data));
    }

    println!("OK — {} symlinks ready in {}", made, link_dir.display());
    Ok(())
}

fn env_cfg() -> Result<(String, String, PathBuf, String)> {
    if get_current_uid() != 0 { return Err(Error::NotRoot); }

    let real_user = env::var("SUDO_USER").ok().filter(|s| !s.is_empty())
            .or_else(|| get_user_by_uid(get_current_uid())
                .map(|u| u.name().to_string_lossy().into_owned()))
        .ok_or(Error::NoUser)?;

    let home = get_user_by_name(&real_user).map(|u| u.home_dir().to_path_buf())
        .ok_or(Error::NoHome(real_user.clone()))?;

    if !has_command("setfacl") || !has_command("getfacl") {
        return Err(Error::NeedAcl);
    }

    let acl_test = Command::new("getfacl")
        .arg("--no-effective")
        .arg("--absolute-names")
        .arg(&home)
        .output();
    
    match acl_test {
        Ok(output) if output.status.success() => {
        }
        Ok(output) if !output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Operation not supported") || stderr.contains("not supported") {
                return Err(Error::AclDisabled);
            }
        }
        Err(e) => { return Err(Error::AclVerify(e)); }
        _ => {}
    }

    let api_key = env::var("API_KEY").or_else(|_| env::var("PTERO_API_KEY")).map_err(|_| Error::NoApiKey)?;
    let mut panel = env::var("PANEL_FQDN").or_else(|_| env::var("PTERO_PANEL")).map_err(|_| Error::NoPanel)?;
    if !(panel.starts_with("http://") || panel.starts_with("https://")) { return Err(Error::BadPanel); }
    while panel.ends_with('/') { panel.pop(); }

    Ok((api_key, panel, home, real_user))
}

// ── Config ──────────────────────────────────────────────────────────────────

fn read_wings_config(p: impl AsRef<Path>) -> Result<WingsConfig> {
    let mut s = String::new();
    fs::File::open(p.as_ref())?.read_to_string(&mut s)?;
    let cfg: WingsConfig = serde_yaml_ng::from_str(&s)?;
    if cfg.uuid.len() < 8 { return Err(Error::BadConfig("bad node uuid in config.yml")); }
    if cfg.system.data.trim().is_empty() { return Err(Error::BadConfig("missing system.data in config.yml")); }
    Ok(cfg)
}

// ── API ─────────────────────────────────────────────────────────────────────

fn fetch_node_id(panel: &str, key: &str, uuid: &str) -> Result<u64> {
    let url = format!("{}/api/application/nodes", panel);
    let mut response = ureq::get(&url)
        .header("Authorization", format!("Bearer {}", key))
        .header("Accept", "application/vnd.pterodactyl.v1+json")
        .query("filter[uuid]", uuid)
        .query("per_page", "1")
        .call()?;
    let js: Json = response.body_mut().read_json()?;
    js["data"][0]["attributes"]["id"].as_u64().ok_or(Error::NodeNotFound)
}

fn fetch_servers_on_node(panel: &str, key: &str, node_id: u64) -> Result<Vec<Server>> {
    let base = format!("{}/api/application/servers", panel);
    let mut page = 1u32; let mut out = Vec::new();
    loop {
        let mut response = ureq::get(&base)
            .header("Authorization", format!("Bearer {}", key))
            .header("Accept", "application/vnd.pterodactyl.v1+json")
            .query("per_page", "100").query("page", &page.to_string())
            .call()?;
        let js: Json = response.body_mut().read_json()?;
        if let Some(arr) = js["data"].as_array() {
            for s in arr {
                let a = &s["attributes"]; if a["node"].as_u64() != Some(node_id) { continue; }
                if let (Some(u), Some(n)) = (a["uuid"].as_str(), a["name"].as_str()) {
                    out.push(Server { uuid: u.to_string(), name: n.to_string() });
                }
            }
        }
        let cur = js["meta"]["pagination"]["current_page"].as_u64().unwrap_or(page as u64);
        let tot = js["meta"]["pagination"]["total_pages"].as_u64().unwrap_or(cur);
        if cur >= tot { break; }
        page += 1;
    }
    Ok(out)
}

// ── Filesystem Utilities ────────────────────────────────────────────────────

fn prune_dangling(dir: &Path) -> Result<()> {
    for e in fs::read_dir(dir)? { let p = e?.path(); if p.symlink_metadata()?.file_type().is_symlink() { if let Ok(t) = fs::read_link(&p) { if !t.exists() { let _ = fs::remove_file(&p); } } } }
    Ok(())
}

// ── Group & ACL Management ──────────────────────────────────────────────────

fn group_management(gid: u32, username: &str, link_dir: &Path, data_dir: &Path) {
    use uzers::get_group_by_gid;

    if let Some(u) = get_user_by_name(username) {
        let uname_os: OsString = u.name().to_os_string();
        let uname_disp = u.name().to_string_lossy();

        let Some(gname) = get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().into_owned())
        else {
            eprintln!("warning: group with gid {gid} not found; cannot modify membership.");
            return;
        };

        let mut needs_group_add = false;
        let mut needs_acl_fix = false;

        if let Some(gs) = get_user_groups(&uname_os, u.primary_group_id()) {
            if !gs.iter().any(|g| g.gid() == gid) {
                needs_group_add = true;
            }
        }

        let paths = vec![link_dir.to_path_buf(), data_dir.to_path_buf()];
        if !check_group_acl_permissions(&paths, &gname) {
            needs_acl_fix = true;
        }

        if needs_group_add || needs_acl_fix {
            let prompt = match (needs_group_add, needs_acl_fix) {
                (true, true) => {
                    eprintln!("note: '{}' is not in group '{}' (gid {gid}) and ACL permissions are missing.", uname_disp, gname);
                    "Add user to group and set ACL permissions? [y/N] "
                }
                (true, false) => {
                    eprintln!("note: '{}' is not in group '{}' (gid {gid}).", uname_disp, gname);
                    "Add user to group? [y/N] "
                }
                (false, true) => {
                    eprintln!("note: ACL permissions for group '{}' are not properly set.", gname);
                    "Set ACL permissions for group access? [y/N] "
                }
                (false, false) => unreachable!(),
            };

            eprint!("{}", prompt);
            let _ = io::stderr().flush();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_ok() {
                let ans = input.trim().to_ascii_lowercase();
                if ans == "y" || ans == "yes" {
                    if needs_group_add {
                        match Command::new("usermod")
                            .arg("-a")
                            .arg("-G")
                            .arg(&gname)
                            .arg(&uname_os)
                            .status()
                        {
                            Ok(s) if s.success() => {
                                eprintln!("✔ added '{}' to group '{}'. You may need to re-login or run 'newgrp {}' to apply it.", uname_disp, gname, gname);
                            }
                            Ok(s) => {
                                eprintln!("error: usermod exited with status {}", s);
                                return;
                            }
                            Err(e) => {
                                eprintln!("error: failed to run usermod: {e}");
                                return;
                            }
                        }
                    }

                    if let Err(e) = set_group_acl_permissions(&paths, &gname) {
                        eprintln!("warning: failed to set ACL permissions: {}", e);
                    } else {
                        eprintln!("✔ ACL permissions set for group '{}'.", gname);
                    }
                } else {
                    eprintln!("skipped group and ACL setup.");
                }
            }
        }
    }
}

fn check_group_acl_permissions(paths: &[PathBuf], gname: &str) -> bool {
    for dir in collect_dirs_and_ancestors(paths) {
        let output = match Command::new("getfacl")
            .arg("--no-effective")
            .arg("--absolute-names")
            .arg(&dir)
            .output()
        {
            Ok(o) if o.status.success() => o,
            Ok(_) => {
                eprintln!("warning: getfacl failed for {}", dir.display());
                return false;
            }
            Err(e) => {
                eprintln!("warning: failed to get ACL for {}: {e}", dir.display());
                return false;
            }
        };

        let acl_output = String::from_utf8_lossy(&output.stdout);
        let expected_entry = format!("group:{}:r-x", gname);

        if !acl_output.lines().any(|line| line.trim() == expected_entry) {
            eprintln!("warning: missing ACL entry '{}' on {}", expected_entry, dir.display());
            return false;
        }
    }

    true
}

fn set_group_acl_permissions(paths: &[PathBuf], gname: &str) -> Result<()> {
    for dir in collect_dirs_and_ancestors(paths) {
        let status = Command::new("setfacl")
            .arg("-m")
            .arg(format!("g:{}:rx", gname))
            .arg(&dir)
            .status()
            .map_err(|e| Error::SetAcl { dir: dir.clone(), source: e })?;

        if !status.success() {
            eprintln!("warning: setfacl failed for {}", dir.display());
        }
    }

    Ok(())
}