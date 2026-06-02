use std::process::Command;

fn run_quantix_without_vision_provider(args: &[&str]) -> (String, String, bool) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_quantix"));
    command.args(args);

    for key in [
        "DEEPSEEK_API_KEY",
        "DEEPSEEK_BASE_URL",
        "DEEPSEEK_VISION_MODEL",
        "OPENAI_API_KEY",
        "OPENAI_BASE_URL",
        "OPENAI_VISION_MODEL",
    ] {
        command.env_remove(key);
    }

    let output = command.output().expect("failed to run quantix");

    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.success(),
    )
}

fn write_tiny_image_fixture(name: &str, extension: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "quantix-import-image-cli-{name}-{}-{}.{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default(),
        extension
    ));
    std::fs::write(&path, [0_u8]).expect("failed to write image fixture");
    path
}

fn write_tiny_png_fixture(name: &str) -> std::path::PathBuf {
    write_tiny_image_fixture(name, "png")
}

#[test]
fn import_from_image_fails_closed_for_unsupported_image_format() {
    let image = write_tiny_image_fixture("unsupported-format", "bmp");
    let image_path = image.to_string_lossy().to_string();

    let (stdout, stderr, success) = run_quantix_without_vision_provider(&[
        "import",
        "from-image",
        "--file",
        &image_path,
        "--model",
        "deepseek",
    ]);

    let _ = std::fs::remove_file(image);

    assert!(
        !success,
        "expected import from-image to reject unsupported image format, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no image-recognition output before image format validation failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("Unsupported"),
        "expected Unsupported error kind for image format boundary, stderr={stderr}"
    );
    assert!(
        stderr.contains("image format 不支持"),
        "expected stable unsupported image format boundary text, stderr={stderr}"
    );
    assert!(
        stderr.contains("bmp") && stderr.contains("png, jpg, jpeg, gif, webp"),
        "expected rejected extension and supported image formats in stderr, stderr={stderr}"
    );
    assert!(
        !stderr.contains("Vision provider 尚未配置"),
        "expected image format validation to fail before Vision provider config validation, stderr={stderr}"
    );
}

#[test]
fn import_from_image_fails_closed_when_deepseek_key_missing() {
    let image = write_tiny_png_fixture("deepseek-missing-key");
    let image_path = image.to_string_lossy().to_string();

    let (stdout, stderr, success) = run_quantix_without_vision_provider(&[
        "import",
        "from-image",
        "--file",
        &image_path,
        "--model",
        "deepseek",
    ]);

    let _ = std::fs::remove_file(image);

    assert!(
        !success,
        "expected import from-image to fail without DeepSeek Vision config, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no placeholder image-recognition output before config failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("Vision provider 尚未配置"),
        "expected Vision provider config boundary in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("DEEPSEEK_API_KEY"),
        "expected missing DeepSeek API key guidance in stderr, stderr={stderr}"
    );
}

#[test]
fn import_from_image_openai_model_requires_openai_key() {
    let image = write_tiny_png_fixture("openai-missing-key");
    let image_path = image.to_string_lossy().to_string();

    let (stdout, stderr, success) = run_quantix_without_vision_provider(&[
        "import",
        "from-image",
        "--file",
        &image_path,
        "--model",
        "openai",
    ]);

    let _ = std::fs::remove_file(image);

    assert!(
        !success,
        "expected import from-image --model openai to fail without OpenAI Vision config, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no placeholder image-recognition output before openai config failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("Vision provider 尚未配置"),
        "expected Vision provider config boundary in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("OPENAI_API_KEY"),
        "expected missing OpenAI API key guidance in stderr, stderr={stderr}"
    );
}
