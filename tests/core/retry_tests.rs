use mc_server_wrapper_core::utils::retry_async;
use std::time::Duration;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use anyhow::{anyhow, Result};

#[tokio::test]
async fn test_retry_async_success() -> Result<()> {
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();

    let result = retry_async(
        || {
            let a = attempts_clone.clone();
            async move {
                let count = a.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(anyhow!("Failed"))
                } else {
                    Ok("Success")
                }
            }
        },
        3,
        Duration::from_millis(10),
        "test_operation"
    ).await?;

    assert_eq!(result, "Success");
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
    Ok(())
}

#[tokio::test]
async fn test_retry_async_failure() -> Result<()> {
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();

    let result = retry_async(
        || {
            let a = attempts_clone.clone();
            async move {
                a.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(anyhow!("Permanent Failure"))
            }
        },
        3,
        Duration::from_millis(10),
        "test_operation"
    ).await;

    assert!(result.is_err());
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("Permanent Failure"));
    assert!(err_msg.contains("test_operation failed after 3 attempts"));
    Ok(())
}
