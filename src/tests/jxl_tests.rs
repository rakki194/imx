#![warn(clippy::all, clippy::pedantic)]

use crate::jxl;
use std::fs;
use std::future::Future;
use std::io::Write;
use std::pin::Pin;
use tempfile::TempDir;

#[tokio::test]
async fn test_is_jxl_file() {
    assert!(jxl::is_jxl_file(std::path::Path::new("test.jxl")));
    assert!(jxl::is_jxl_file(std::path::Path::new("test.JXL")));
    assert!(!jxl::is_jxl_file(std::path::Path::new("test.png")));
    assert!(!jxl::is_jxl_file(std::path::Path::new("test")));
}

#[tokio::test]
async fn test_process_jxl_file_invalid_extension() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let invalid_file = temp_dir.path().join("test.png");
    fs::write(&invalid_file, b"not a jxl file")?;

    let result = jxl::process_jxl_file::<
        fn(&std::path::Path) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>,
        Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>,
    >(&invalid_file, None)
    .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Not a JXL file"));
    Ok(())
}

#[tokio::test]
async fn test_process_jxl_file_with_processor() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let jxl_file = temp_dir.path().join("test.jxl");
    fs::write(&jxl_file, b"dummy jxl data")?;

    let processed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let processed_clone = processed.clone();

    let processor = move |path: &std::path::Path| {
        let path = path.to_owned();
        let processed = processed_clone.clone();
        Box::pin(async move {
            // Create a PNG file since JXL conversion will fail
            let mut file = fs::File::create(&path)?;
            file.write_all(b"dummy png data")?;

            assert_eq!(path.extension().unwrap(), "png");
            processed.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        })
    };

    // The JXL processing will fail, but the processor should still be called
    let result = jxl::process_jxl_file(&jxl_file, Some(processor)).await;
    assert!(result.is_err()); // Should fail due to invalid JXL data
    assert!(processed.load(std::sync::atomic::Ordering::SeqCst)); // But processor should still be called
    Ok(())
}
