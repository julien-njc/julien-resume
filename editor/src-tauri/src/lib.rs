use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::Manager;

// ── Embedded build resources ──────────────────────────────────────────────────
// Compiled into the binary so the app is self-contained: no resource_dir lookup,
// works identically in `tauri dev` and a production bundle.
//
// CARGO_MANIFEST_DIR is set by Cargo at compile time to the directory of
// Cargo.toml (editor/src-tauri/), so ../../ reaches the repo root.

macro_rules! src {
    ($path:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../", $path))
    };
}

static DOCKERFILE: &str = src!("Dockerfile");
static BUILD_SCRIPT: &str = src!("scripts/build-resume.sh");
static RESUME_MODEL: &str = src!("scripts/resume_model.py");
static RENDER_RESUME: &str = src!("scripts/render_resume.py");
static BUILD_ATS_DOCX: &str = src!("scripts/build_ats_docx.py");
static CSS_ATS: &str = src!("resume-ats.css");
static CSS_STYLED: &str = src!("resume-stylish.css");

// ── Persisted app config ─────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Default)]
struct AppConfig {
    data_folder: Option<String>,
}

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("config.json"))
}

#[tauri::command]
fn load_app_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let path = config_path(&app)?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_app_config(app: tauri::AppHandle, data_folder: String) -> Result<(), String> {
    let path = config_path(&app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let config = AppConfig { data_folder: Some(data_folder) };
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

// ── Folder validation ─────────────────────────────────────────────────────────

fn validate_folder(folder: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(folder);
    if path.is_dir() {
        Ok(path)
    } else {
        Err(format!("not a valid directory: {folder}"))
    }
}

// ── resume.json ───────────────────────────────────────────────────────────────

#[tauri::command]
fn load_resume(data_folder: String) -> Result<String, String> {
    let folder = validate_folder(&data_folder)?;
    let path = folder.join("resume.json");
    if !path.exists() {
        return Ok("{}".to_string());
    }
    fs::read_to_string(&path).map_err(|e| format!("failed to read resume.json: {e}"))
}

#[tauri::command]
fn save_resume(data_folder: String, contents: String) -> Result<(), String> {
    let folder = validate_folder(&data_folder)?;
    let _: serde_json::Value =
        serde_json::from_str(&contents).map_err(|e| format!("invalid JSON: {e}"))?;
    fs::write(folder.join("resume.json"), contents)
        .map_err(|e| format!("failed to write resume.json: {e}"))
}

// ── applications.json ─────────────────────────────────────────────────────────

#[tauri::command]
fn load_applications(data_folder: String) -> Result<String, String> {
    let folder = validate_folder(&data_folder)?;
    let path = folder.join("applications.json");
    if !path.exists() {
        return Ok(r#"{"applications":[]}"#.to_string());
    }
    fs::read_to_string(&path).map_err(|e| format!("failed to read applications.json: {e}"))
}

#[tauri::command]
fn save_applications(data_folder: String, contents: String) -> Result<(), String> {
    let folder = validate_folder(&data_folder)?;
    let _: serde_json::Value =
        serde_json::from_str(&contents).map_err(|e| format!("invalid JSON: {e}"))?;
    fs::write(folder.join("applications.json"), contents)
        .map_err(|e| format!("failed to write applications.json: {e}"))
}

// ── Docker build ──────────────────────────────────────────────────────────────

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

fn generated_file_candidates(root: &Path) -> Vec<(String, PathBuf)> {
    let build = root.join("build");
    vec![
        ("ATS DOCX".into(), build.join("Julien_Pireaud_Resume_ATS.docx")),
        ("ATS PDF".into(), build.join("Julien_Pireaud_Resume_ATS.pdf")),
        ("Styled PDF".into(), build.join("Julien_Pireaud_Resume.pdf")),
        ("ATS Markdown".into(), build.join("Julien_Pireaud_Resume_ATS.md")),
        ("Styled Markdown".into(), build.join("Julien_Pireaud_Resume_Styled.md")),
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
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| canonical.to_string_lossy().into_owned());
            Some(GeneratedFile { label, path: canonical.to_string_lossy().into_owned(), relative_path })
        })
        .collect()
}

fn open_with_default_app(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut cmd = { let mut c = Command::new("open"); c.arg(path); c };
    #[cfg(target_os = "linux")]
    let mut cmd = { let mut c = Command::new("xdg-open"); c.arg(path); c };
    #[cfg(target_os = "windows")]
    let mut cmd = { let mut c = Command::new("cmd"); c.args(["/C", "start", "", &path.to_string_lossy()]); c };

    cmd.status()
        .map_err(|e| format!("failed to open file: {e}"))
        .and_then(|s| if s.success() { Ok(()) } else { Err(format!("opener exited {:?}", s.code())) })
}

#[tauri::command]
fn list_generated_files(data_folder: String) -> Result<Vec<GeneratedFile>, String> {
    let folder = validate_folder(&data_folder)?;
    Ok(collect_generated_files(&folder))
}

#[tauri::command]
fn open_generated_file(data_folder: String, path: String) -> Result<(), String> {
    let folder = validate_folder(&data_folder)?;
    let canonical = PathBuf::from(&path)
        .canonicalize()
        .map_err(|e| format!("failed to resolve file: {e}"))?;
    let build_root = folder
        .join("build")
        .canonicalize()
        .map_err(|e| format!("failed to resolve build dir: {e}"))?;
    if !canonical.starts_with(&build_root) {
        return Err("can only open files inside the build directory".into());
    }
    open_with_default_app(&canonical)
}

/// Writes the embedded Dockerfile, scripts, and CSS into `data_folder/.resume-builder/`,
/// then generates a `compose.yaml` inside that same directory so Docker's working
/// directory and volume paths are all self-contained.
fn prepare_build_context(folder: &Path) -> Result<PathBuf, String> {
    let ctx = folder.join(".resume-builder");
    let scripts = ctx.join("scripts");
    fs::create_dir_all(&scripts)
        .map_err(|e| format!("failed to create .resume-builder/scripts: {e}"))?;

    let write = |path: &PathBuf, content: &str| -> Result<(), String> {
        fs::write(path, content).map_err(|e| format!("failed to write {}: {e}", path.display()))
    };

    write(&ctx.join("Dockerfile"), DOCKERFILE)?;
    write(&scripts.join("build-resume.sh"), BUILD_SCRIPT)?;
    write(&scripts.join("resume_model.py"), RESUME_MODEL)?;
    write(&scripts.join("render_resume.py"), RENDER_RESUME)?;
    write(&scripts.join("build_ats_docx.py"), BUILD_ATS_DOCX)?;
    write(&ctx.join("resume-ats.css"), CSS_ATS)?;
    write(&ctx.join("resume-stylish.css"), CSS_STYLED)?;

    // compose.yaml lives inside .resume-builder/ so `.` refers to that directory.
    // The data folder is mounted via its absolute path.
    let data_abs = folder
        .canonicalize()
        .map_err(|e| format!("failed to resolve data folder: {e}"))?;

    let compose = format!(
        "services:\n\
         \x20 resume-builder:\n\
         \x20   build:\n\
         \x20     context: .\n\
         \x20     dockerfile: Dockerfile\n\
         \x20   volumes:\n\
         \x20     - {data}:/workspace\n\
         \x20     - ./scripts:/workspace/scripts\n\
         \x20     - ./resume-ats.css:/workspace/resume-ats.css\n\
         \x20     - ./resume-stylish.css:/workspace/resume-stylish.css\n\
         \x20   working_dir: /workspace\n\
         \x20   command: sh scripts/build-resume.sh\n",
        data = data_abs.display()
    );
    write(&ctx.join("compose.yaml"), &compose)?;

    Ok(ctx)
}

#[tauri::command]
async fn build_resume(data_folder: String) -> Result<BuildResult, String> {
    let folder = validate_folder(&data_folder)?;
    let ctx = prepare_build_context(&folder)?;

    tauri::async_runtime::spawn_blocking(move || {
        let output = Command::new("docker")
            .args(["compose", "run", "--rm", "resume-builder"])
            .current_dir(&ctx)
            // Docker Desktop on macOS is not on the default GUI-app PATH.
            .env("PATH", "/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin")
            .output()
            .map_err(|e| format!("failed to run docker: {e}"))?;
        Ok(BuildResult {
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            generated_files: collect_generated_files(&folder),
        })
    })
    .await
    .map_err(|e| format!("build task failed: {e}"))?
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            load_app_config,
            save_app_config,
            load_resume,
            save_resume,
            load_applications,
            save_applications,
            list_generated_files,
            open_generated_file,
            build_resume,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
