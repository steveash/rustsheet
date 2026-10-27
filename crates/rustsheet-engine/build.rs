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

/// The files whose change can change `git describe`: this worktree's HEAD, the branch ref it
/// points at, `packed-refs` and the tags. Cargo resolves a relative `rerun-if-changed` path
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
    let mut paths = vec![
        git_dir.join("HEAD"),
        common_dir.join("packed-refs"),
        common_dir.join("refs").join("tags"),
    ];
    if let Some(branch) = run_git(&["symbolic-ref", "-q", "HEAD"]) {
        // A packed branch has no loose ref file until its next commit writes one: watch the
        // directory that commit will write to instead.
        let loose = common_dir.join(&branch);
        paths.push(match loose.parent() {
            Some(dir) if !loose.exists() => dir.to_path_buf(),
            _ => loose,
        });
    }
    paths.retain(|p| p.exists());
    paths
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    for path in watched_git_paths() {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    // Prefer exact tag
    let version = run_git(&["describe", "--tags", "--exact-match"])
        .or_else(|| run_git(&["describe", "--tags", "--dirty", "--always"]))
        .unwrap_or_else(|| "unknown".into());

    println!("cargo:rustc-env=GIT_VERSION={version}");
}
