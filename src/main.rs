use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use serde_json::Value as Json;
use std::env;
use std::fs;
use std::io::Read;
use std::os::unix::fs as unix_fs;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use uzers::{get_current_uid, get_user_by_name, get_user_by_uid, get_user_groups, os::unix::UserExt};

#[cfg(not(target_os = "linux"))]
compile_error!("Linux-only");

static DEFAULT_WINGS_CONFIG: &str = "/etc/pterodactyl/config.yml";

#[derive(Deserialize)]
struct WingsConfig { uuid: String, system: System }
#[derive(Deserialize)]
struct System { data: String, user: SystemUser }
#[derive(Deserialize)]
struct SystemUser { gid: Option<u32> }

#[derive(Clone, Debug)]
struct Server { uuid: String, name: String }

fn main() { if let Err(e) = run() { eprintln!("{e:#}"); std::process::exit(1); } }

fn run() -> Result<()> {
    let (api_key, panel, home) = env_cfg()?;
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
        unix_fs::symlink(&target, &link).with_context(|| format!("symlink {} -> {}", link.display(), target.display()))?;
        made += 1;
    }

    if let Some(gid) = system.user.gid { group_management(gid); }

    println!("OK — {} symlinks ready in {}", made, link_dir.display());
    Ok(())
}

fn env_cfg() -> Result<(String, String, PathBuf)> {
    if get_current_uid() != 0 { bail!("must run as root/sudo"); }

    let real_user = env::var("SUDO_USER").ok().filter(|s| !s.is_empty())
        .or_else(|| get_user_by_uid(get_current_uid()).map(|u| u.name().to_string_lossy().into_owned()))
        .ok_or_else(|| anyhow!("cannot resolve real user"))?;

    let home = get_user_by_name(&real_user)
        .map(|u| u.home_dir().to_path_buf())
        .ok_or_else(|| anyhow!("home dir not found for {real_user}"))?;

    let api_key = env::var("API_KEY").or_else(|_| env::var("PTERO_API_KEY")).context("API key missing")?;
    let mut panel = env::var("PANEL_FQDN").or_else(|_| env::var("PTERO_PANEL")).context("panel URL missing")?;
    if !(panel.starts_with("http://") || panel.starts_with("https://")) { bail!("panel must start with http:// or https://"); }
    while panel.ends_with('/') { panel.pop(); }

    Ok((api_key, panel, home))
}

fn read_wings_config(p: impl AsRef<Path>) -> Result<WingsConfig> {
    let mut s = String::new();
    fs::File::open(p.as_ref())?.read_to_string(&mut s)?;
    let cfg: WingsConfig = serde_yaml_ng::from_str(&s)?;
    if cfg.uuid.len() < 8 { bail!("bad node uuid in config.yml"); }
    if cfg.system.data.trim().is_empty() { bail!("missing system.data in config.yml"); }
    Ok(cfg)
}

fn fetch_node_id(panel: &str, key: &str, uuid: &str) -> Result<u64> {
    let url = format!("{}/api/application/nodes", panel);
    let mut response = ureq::get(&url)
        .header("Authorization", format!("Bearer {}", key))
        .header("Accept", "application/vnd.pterodactyl.v1+json")
        .query("filter[uuid]", uuid)
        .query("per_page", "1")
        .call()?;
    let js: Json = response.body_mut().read_json()?;
    js["data"][0]["attributes"]["id"].as_u64().ok_or_else(|| anyhow!("node not found"))
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

fn prune_dangling(dir: &Path) -> Result<()> {
    for e in fs::read_dir(dir)? { let p = e?.path(); if p.symlink_metadata()?.file_type().is_symlink() { if let Ok(t) = fs::read_link(&p) { if !t.exists() { let _ = fs::remove_file(&p); } } } }
    Ok(())
}

fn safe(s: &str) -> String { s.chars().map(|c| if c.is_ascii_alphanumeric() || "-_. ".contains(c) { c } else { '_' }).collect() }

fn group_management(gid: u32) {
    use std::io::{self, Write};
    use std::process::Command;
    use uzers::get_group_by_gid;

    if let Some(u) = get_user_by_uid(get_current_uid()) {
        let uname_os: OsString = u.name().to_os_string();
        let uname_disp = u.name().to_string_lossy();

        if let Some(gs) = get_user_groups(&uname_os, u.primary_group_id()) {
            if !gs.iter().any(|g| g.gid() == gid) {
                let Some(gname) = get_group_by_gid(gid)
                    .map(|g| g.name().to_string_lossy().into_owned())
                else {
                    eprintln!("warning: group with gid {gid} not found; cannot modify membership.");
                    return;
                };

                eprintln!("note: '{}' is not in group '{}' (gid {gid}).", uname_disp, gname);
                eprint!("Add now? [y/N] ");
                let _ = io::stderr().flush();

                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_ok() {
                    let ans = input.trim().to_ascii_lowercase();
                    if ans == "y" || ans == "yes" {
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
                            }
                            Err(e) => {
                                eprintln!("error: failed to run usermod: {e}");
                            }
                        }
                    } else {
                        eprintln!("skipped adding user to group.");
                    }
                }
            }
        }
    }
}
