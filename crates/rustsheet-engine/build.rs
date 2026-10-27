// At build time, we grab the release version from the git tag.
// This is used by INFO("release")
use std::path::{Path, PathBuf};
use std::process::Command;

fn run_git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// The files whose change can change `git describe`: this worktree's HEAD and reflog (or, with
/// reflogs off, the branch ref HEAD points at), `packed-refs` and the tags. Cargo resolves a relative `rerun-if-changed` path
/// against the crate directory, where `.git` does not exist, and a missing watched file always
/// counts as changed, so the script re-ran on every cargo call (T-2755). Ask git for its real
/// directories (`.git` is a file in a worktree) and watch only paths that exist.
fn watched_git_paths() -> Vec<PathBuf> {
    let Some(dirs) = run_git(&[
        "rev-parse",
        "--path-format=absolute",
        "--git-dir",
        "--git-common-dir",
    ]) else {
        return Vec::new();
    };
    let mut lines = dirs.lines();
    let (Some(git_dir), Some(common_dir)) = (lines.next(), lines.next()) else {
        return Vec::new();
    };
    let (git_dir, common_dir) = (Path::new(git_dir), Path::new(common_dir));
    // `--path-format` needs git 2.31 or newer; an older git echoes the unknown option as its
    // first line. Without two absolute directories, watch only build.rs: GIT_VERSION may then go
    // stale, but the engine never rebuilds on every call.
    if !git_dir.is_absolute() || !common_dir.is_absolute() {
        return Vec::new();
    }
    // `packed-refs` and `refs/tags` are dropped below if missing now; packing refs later also
    // changes the watched ref files and directories, so the rerun still happens.
    let mut paths = vec![
        git_dir.join("HEAD"),
        common_dir.join("packed-refs"),
        common_dir.join("refs").join("tags"),
    ];
    // This worktree's reflog gains a line on every commit, reset, rebase and checkout here, and
    // not when other branches move, so it is the cheapest exact signal. Reflogs can be off.
    let reflog = git_dir.join("logs").join("HEAD");
    if reflog.exists() {
        paths.push(reflog);
    } else if let Some(branch) = run_git(&["symbolic-ref", "-q", "HEAD"]) {
        // A packed branch has no loose ref file until its next commit writes one, and packing
        // also removes the empty directories of a slashed name (`agent/<lane>/T-...`): watch the
        // nearest existing directory on the way to that file, whose entries the commit changes.
        let loose = common_dir.join(&branch);
        let watched = loose
            .ancestors()
            .take_while(|p| p.starts_with(common_dir))
            .find(|p| p.exists())
            .map(Path::to_path_buf);
        paths.extend(watched);
    }
    paths.retain(|p| p.exists());
    paths
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    for path in watched_git_paths() {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    // The work tree is not watched (that would rebuild on every edit), so `-dirty` is only as
    // fresh as the last HEAD or ref change; it was exact while the script re-ran on every call.
    // Prefer exact tag
    let version = run_git(&["describe", "--tags", "--exact-match"])
        .or_else(|| run_git(&["describe", "--tags", "--dirty", "--always"]))
        .unwrap_or_else(|| "unknown".into());

    println!("cargo:rustc-env=GIT_VERSION={version}");
}
