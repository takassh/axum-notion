use anyhow::Context;
use axum::Json;
use std::{fs::File, io::Write, process::Command};
use tempfile::{tempdir, NamedTempFile};
pub mod request;
pub mod response;

use crate::response::{ApiResponse, IntoApiResponse};

use self::{request::PostCodeRequest, response::PostCodeResp};

pub async fn post_code(
    Json(payload): Json<PostCodeRequest>,
) -> ApiResponse<Json<PostCodeResp>> {
    let dir = tempdir()
        .context("failed to create tmp dir")
        .into_response("502-008")?;
    let file_path = dir.path().join("main.rs");
    let mut file = File::create(file_path.clone())
        .context("failed to create tmp file")
        .into_response("502-008")?;

    writeln!(file, "{}", payload.code)
        .context("failed to write code to tmp file")
        .into_response("502-008")?;
    let mut file = NamedTempFile::new()
        .context("failed to create tmp file")
        .into_response("502-008")?;
    writeln!(file, "{}", payload.code)
        .context("failed to write code to tmp file")
        .into_response("502-008")?;

    let output = Command::new("rustc")
        .args(["./main.rs"])
        .current_dir(dir.path())
        .output()
        .context("failed to compile code")
        .into_response("502-008")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "failed to compile: {}",
            String::from_utf8_lossy(&output.stdout)
        ))
        .into_response("502-008");
    }

    let output = Command::new("./main")
        .current_dir(dir.path())
        .output()
        .context("failed to run code")
        .into_response("502-008")?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result = if output.status.success() {
        stdout.into()
    } else {
        stderr.into()
    };

    drop(file);
    dir.close()
        .context("failed to close dir")
        .into_response("502-008")?;

    Ok(PostCodeResp { result }.into())
}
