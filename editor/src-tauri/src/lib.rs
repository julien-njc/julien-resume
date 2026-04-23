use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Serialize)]
struct BuildResult {
    status: i32,
    stdout: String,
    stderr: String,
    generated_files: Vec<GeneratedFile>,
}

#[derive(Serialize)]
struct GeneratedFile {
    label: String,
    path: String,
    relative_path: String,
}

fn is_resume_root(path: &Path) -> bool {
    path.join("resume.json").is_file() && path.join("compose.yaml").is_file()
}

fn search_upwards(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(path) = current {
        if is_resume_root(path) {
            return Some(path.to_path_buf());
        }
        current = path.parent();
    }
    None
}

fn default_root_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.to_path_buf());
        }
    }
    candidates
}

fn resolve_repo_root(explicit: Option<String>) -> Result<PathBuf, String> {
    if let Some(value) = explicit {
        let path = PathBuf::from(value);
        let canonical = path
            .canonicalize()
            .map_err(|err| format!("failed to resolve repo root: {err}"))?;
        if is_resume_root(&canonical) {
            return Ok(canonical);
        }
        return Err("repo root must contain resume.json and compose.yaml".into());
    }

    for candidate in default_root_candidates() {
        if let Some(found) = search_upwards(&candidate) {
            return Ok(found);
        }
    }

    Err("could not find the resume repository root".into())
}

fn resume_json_path(root: &Path) -> PathBuf {
    root.join("resume.json")
}

fn generated_file_candidates(root: &Path) -> Vec<(String, PathBuf)> {
    let build = root.join("build");
    vec![
        (
            "ATS DOCX".into(),
            build.join("Julien_Pireaud_Resume_ATS.docx"),
        ),
        (
            "ATS PDF".into(),
            build.join("Julien_Pireaud_Resume_ATS.pdf"),
        ),
        (
            "Styled PDF".into(),
            build.join("Julien_Pireaud_Resume.pdf"),
        ),
        (
            "ATS Markdown".into(),
            build.join("Julien_Pireaud_Resume_ATS.md"),
        ),
        (
            "Styled Markdown".into(),
            build.join("Julien_Pireaud_Resume_Styled.md"),
        ),
    ]
}

fn collect_generated_files(root: &Path) -> Vec<GeneratedFile> {
    generated_file_candidates(root)
        .into_iter()
        .filter_map(|(label, path)| {
            if !path.is_file() {
                return None;
            }
            let canonical = path.canonicalize().ok()?;
            let relative_path = canonical
                .strip_prefix(root)
                .ok()
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_else(|| canonical.to_string_lossy().into_owned());
            Some(GeneratedFile {
                label,
                path: canonical.to_string_lossy().into_owned(),
                relative_path,
            })
        })
        .collect()
}

fn open_with_default_app(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut cmd = Command::new("open");
        cmd.arg(path);
        cmd
    };

    #[cfg(target_os = "linux")]
    let mut command = {
        let mut cmd = Command::new("xdg-open");
        cmd.arg(path);
        cmd
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "start", "", &path.to_string_lossy()]);
        cmd
    };

    command
        .status()
        .map_err(|err| format!("failed to open file: {err}"))
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(format!("file opener exited with status {:?}", status.code()))
            }
        })
}

#[tauri::command]
fn default_resume_root() -> Result<String, String> {
    Ok(resolve_repo_root(None)?
        .to_string_lossy()
        .into_owned())
}

#[tauri::command]
fn load_resume(repo_root: String) -> Result<String, String> {
    let root = resolve_repo_root(Some(repo_root))?;
    fs::read_to_string(resume_json_path(&root)).map_err(|err| format!("failed to read resume.json: {err}"))
}

#[tauri::command]
fn save_resume(repo_root: String, contents: String) -> Result<(), String> {
    let root = resolve_repo_root(Some(repo_root))?;
    let _: serde_json::Value =
        serde_json::from_str(&contents).map_err(|err| format!("invalid JSON: {err}"))?;
    fs::write(resume_json_path(&root), contents)
        .map_err(|err| format!("failed to write resume.json: {err}"))
}

#[tauri::command]
fn list_generated_files(repo_root: String) -> Result<Vec<GeneratedFile>, String> {
    let root = resolve_repo_root(Some(repo_root))?;
    Ok(collect_generated_files(&root))
}

#[tauri::command]
fn open_generated_file(repo_root: String, path: String) -> Result<(), String> {
    let root = resolve_repo_root(Some(repo_root))?;
    let canonical = PathBuf::from(path)
        .canonicalize()
        .map_err(|err| format!("failed to resolve generated file: {err}"))?;
    let build_root = root
        .join("build")
        .canonicalize()
        .map_err(|err| format!("failed to resolve build directory: {err}"))?;
    if !canonical.starts_with(&build_root) {
        return Err("can only open files inside the build directory".into());
    }
    open_with_default_app(&canonical)
}

#[tauri::command]
async fn build_resume(repo_root: String) -> Result<BuildResult, String> {
    let root = resolve_repo_root(Some(repo_root))?;

    tauri::async_runtime::spawn_blocking(move || {
        let output = Command::new("docker")
            .args(["compose", "run", "--rm", "resume-builder"])
            .current_dir(&root)
            .output()
            .map_err(|err| format!("failed to run docker compose: {err}"))?;

        Ok(BuildResult {
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            generated_files: collect_generated_files(&root),
        })
    })
    .await
    .map_err(|err| format!("build task failed: {err}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            default_resume_root,
            load_resume,
            save_resume,
            list_generated_files,
            open_generated_file,
            build_resume
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
