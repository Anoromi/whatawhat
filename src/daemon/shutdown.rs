use tokio::select;
use tokio_util::sync::CancellationToken;

/// Detects signals sent to the process. This works with limmited success. 
///
/// On Windows detached processes can't detect signals sent to them, so this should be enhanced in the future to 
/// support another way of sending signals.
pub async fn detect_shutdown(cancelation: CancellationToken) {
    select! {
        _ = tokio::signal::ctrl_c() => {
            cancelation.cancel();
        },
    };
}
