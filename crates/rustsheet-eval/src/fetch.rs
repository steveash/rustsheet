//! `rustsheet-eval fetch`: downloads pinned external suites into `eval/.cache/suites/<name>`.
//!
//! Sources are manifest-declared (trusted repo content), but the downloaded bytes are not:
//! archives are hash-verified before extraction, git sources are pinned to a full commit id,
//! symlinks are never copied and extraction cannot escape the staging directory.

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::suite::{FetchSpec, Suite, FETCH_MARKER};

/// Upper bound for a downloaded archive.
const MAX_ARCHIVE_BYTES: &str = "2000000000";

fn run(cmd: &mut Command) -> Result<()> {
    let desc = format!("{cmd:?}");
    let status = cmd.status().with_context(|| format!("starting {desc}"))?;
    if !status.success() {
        bail!("{desc} failed with {status}");
    }
    Ok(())
}

fn fetch_git(spec: &FetchSpec, rev: &str, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    run(Command::new("git").args(["init", "-q"]).arg(dest))?;
    run(Command::new("git")
        .arg("-C")
        .arg(dest)
        .args(["fetch", "-q", "--depth", "1", "--", &spec.url, rev]))?;
    run(Command::new("git").arg("-C").arg(dest).args([
        "checkout",
        "-q",
        "--detach",
        "FETCH_HEAD",
    ]))?;
    fs::remove_dir_all(dest.join(".git"))?;
    Ok(())
}

fn sha256_hex(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    io::copy(&mut fs::File::open(path)?, &mut hasher)?;
    Ok(hasher.finalize().iter().fold(String::new(), |mut acc, b| {
        let _ = write!(acc, "{b:02x}");
        acc
    }))
}

/// Zip-bomb guard (CLAUDE.md directive 5): the public API's per-package limit, so one
/// definition bounds what the harness inflates from an archive entry.
fn extract_zip(archive: &Path, dest: &Path) -> Result<()> {
    extract_zip_capped(
        archive,
        dest,
        rustsheet::LoadOptions::default().max_uncompressed_bytes,
    )
}

fn extract_zip_capped(archive: &Path, dest: &Path, limit: u64) -> Result<()> {
    let mut zip = zip::ZipArchive::new(fs::File::open(archive)?)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        // `enclosed_name` rejects absolute paths and `..` (zip-slip).
        let name = entry.name()?.into_owned();
        let Some(rel) = entry.enclosed_name() else {
            bail!("unsafe path in archive: {name}");
        };
        let out = dest.join(rel);
        if entry.is_dir() {
            fs::create_dir_all(&out)?;
        } else {
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent)?;
            }
            let copied = io::copy(
                &mut io::Read::take(&mut entry, limit.saturating_add(1)),
                &mut fs::File::create(&out)?,
            )?;
            if copied > limit {
                bail!("archive entry {name} exceeds the {limit}-byte decompressed size limit");
            }
        }
    }
    Ok(())
}

fn fetch_archive(spec: &FetchSpec, sha256: &str, stage: &Path, dest: &Path) -> Result<()> {
    let archive = stage.join("download");
    run(Command::new("curl")
        .args([
            "-fsSL",
            "--proto",
            "=https,file",
            "--max-filesize",
            MAX_ARCHIVE_BYTES,
            "-o",
        ])
        .arg(&archive)
        .args(["--", &spec.url]))?;
    let actual = sha256_hex(&archive)?;
    if !actual.eq_ignore_ascii_case(sha256) {
        bail!(
            "sha256 mismatch for {}: expected {sha256}, got {actual}",
            spec.url
        );
    }
    fs::create_dir_all(dest)?;
    if spec.url.to_ascii_lowercase().ends_with(".zip") {
        extract_zip(&archive, dest)
    } else {
        // GNU/BSD tar both refuse absolute and `..` members by default.
        run(Command::new("tar")
            .arg("-xf")
            .arg(&archive)
            .arg("-C")
            .arg(dest))
    }
}

/// Copies the files selected by `spec` from `src` (the fetched tree) into `out`.
fn select_files(spec: &FetchSpec, src: &Path, out: &Path) -> Result<usize> {
    let root = match &spec.subdir {
        Some(sub) => src.join(sub),
        None => src.to_path_buf(),
    };
    if !root.is_dir() {
        bail!(
            "fetch.subdir {:?} not found in the fetched source",
            spec.subdir
        );
    }
    let include = crate::suite::build_globs(&spec.include)?;
    let mut copied = 0;
    for entry in WalkDir::new(&root).sort_by_file_name() {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue; // directories are recreated on demand; symlinks are never followed or copied
        }
        let rel = entry.path().strip_prefix(&root)?;
        if include.is_match(rel.to_string_lossy().replace('\\', "/").as_str()) {
            let target = out.join(rel);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target)?;
            copied += 1;
        }
    }
    Ok(copied)
}

/// Fetches `suite` into its cache directory, replacing any previous copy.
///
/// # Errors
/// Fails if the suite has no `[fetch]` table, the download or pin verification fails, or the
/// cache cannot be written. A failed fetch leaves the previous cache untouched.
pub fn fetch_suite(suite: &Suite) -> Result<usize> {
    let Some(spec) = &suite.manifest.fetch else {
        bail!("suite {} has no [fetch] table", suite.manifest.name);
    };
    let stage = suite.cache_dir.with_file_name(format!(
        ".partial-{}-{}",
        suite.manifest.name,
        std::process::id()
    ));
    if stage.exists() {
        fs::remove_dir_all(&stage)?;
    }
    fs::create_dir_all(&stage)?;
    let result = fetch_into(spec, &stage, &suite.cache_dir);
    // Best effort: the staging dir is only scratch space.
    let _ = fs::remove_dir_all(&stage);
    result
}

fn fetch_into(spec: &FetchSpec, stage: &Path, cache_dir: &Path) -> Result<usize> {
    let src = stage.join("src");
    match (&spec.rev, &spec.sha256) {
        (Some(rev), _) => fetch_git(spec, rev, &src)?,
        (None, Some(sha)) => fetch_archive(spec, sha, stage, &src)?,
        (None, None) => bail!("fetch spec has no pin"),
    }
    let out = stage.join("out");
    let copied = select_files(spec, &src, &out)?;
    fs::write(out.join(FETCH_MARKER), spec.pin())?;
    if cache_dir.exists() {
        fs::remove_dir_all(cache_dir)?;
    }
    if let Some(parent) = cache_dir.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(&out, cache_dir)?;
    Ok(copied)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use crate::suite::{FetchState, SuiteManifest};

    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("rs-fetch-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    fn write_zip(path: &Path, files: &[(&str, &str)]) {
        let mut zip = zip::ZipWriter::new(fs::File::create(path).expect("create"));
        for (name, body) in files {
            zip.start_file(*name, zip::write::SimpleFileOptions::default())
                .expect("start");
            zip.write_all(body.as_bytes()).expect("write");
        }
        zip.finish().expect("finish");
    }

    #[test]
    fn entry_over_the_size_limit_is_rejected() {
        let dir = scratch("big");
        let zip = dir.join("big.zip");
        write_zip(&zip, &[("pkg/big.bin", &"a".repeat(100))]);
        extract_zip_capped(&zip, &dir.join("ok"), 100).expect("at the limit");
        let err = extract_zip_capped(&zip, &dir.join("bad"), 99).expect_err("over the limit");
        assert!(err.to_string().contains("pkg/big.bin"), "{err}");
    }

    fn suite(dir: &Path, url: &str, sha: &str) -> Suite {
        let text = format!(
            "name = \"ext\"\nkind = \"spec\"\nlicense = \"MIT\"\n[fetch]\nurl = \"{url}\"\nsha256 = \"{sha}\"\nsubdir = \"pkg\"\ninclude = [\"**/*.case.json\"]\n"
        );
        Suite {
            manifest: toml::from_str::<SuiteManifest>(&text).expect("manifest"),
            dir: dir.join("suite"),
            cache_dir: dir.join("cache/ext"),
        }
    }

    #[test]
    fn archive_fetch_verifies_pin_filters_and_marks_fetched() {
        let dir = scratch("ok");
        let zip = dir.join("src.zip");
        write_zip(
            &zip,
            &[
                ("pkg/a/x.case.json", "{}"),
                ("pkg/notes.txt", "skip me"),
                ("other/y.case.json", "{}"),
            ],
        );
        let sha = sha256_hex(&zip).expect("sha");
        let s = suite(&dir, &format!("file://{}", zip.display()), &sha);
        assert_eq!(s.fetch_state(), FetchState::NotFetched);
        assert_eq!(fetch_suite(&s).expect("fetch"), 1);
        assert_eq!(s.fetch_state(), FetchState::Fetched);
        assert!(s.cache_dir.join("a/x.case.json").is_file());
        assert!(!s.cache_dir.join("notes.txt").exists());
        assert_eq!(s.cases().expect("cases").len(), 1);
        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn archive_fetch_rejects_hash_mismatch_and_keeps_old_cache() {
        let dir = scratch("bad");
        let zip = dir.join("src.zip");
        write_zip(&zip, &[("pkg/x.case.json", "{}")]);
        let s = suite(&dir, &format!("file://{}", zip.display()), &"0".repeat(64));
        fs::create_dir_all(&s.cache_dir).expect("mkdir");
        fs::write(s.cache_dir.join("keep"), "old").expect("write");
        let err = fetch_suite(&s).expect_err("mismatch");
        assert!(format!("{err:#}").contains("sha256 mismatch"));
        assert!(s.cache_dir.join("keep").is_file());
        assert_eq!(s.fetch_state(), FetchState::NotFetched);
        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn archive_with_path_traversal_is_rejected() {
        let dir = scratch("slip");
        let zip = dir.join("src.zip");
        write_zip(&zip, &[("../evil.case.json", "{}")]);
        let sha = sha256_hex(&zip).expect("sha");
        let s = suite(&dir, &format!("file://{}", zip.display()), &sha);
        assert!(fetch_suite(&s).is_err());
        assert!(!dir.join("evil.case.json").exists());
        fs::remove_dir_all(&dir).expect("cleanup");
    }
}
