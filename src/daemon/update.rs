use tokio::select;
use tokio_util::sync::CancellationToken;

pub async fn detect_shutdown(cancelation: CancellationToken) {
    select! {
        _ = tokio::signal::ctrl_c() => {
            cancelation.cancel();
        },
    };
}
