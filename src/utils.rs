use crate::error::DashResult;
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub(crate) fn spawn_thread<F>(name: &'static str, interval_minutes: u64, f: F) -> JoinHandle<()>
where
    F: Fn() -> DashResult<()> + Send + 'static,
{
    let duration = Duration::from_secs(interval_minutes * 60);
    thread::spawn(move || loop {
        if let Err(err) = f() {
            error!("the {} thread failed an iteration: {:?}", name, err);
        }
        info!(
            "{} thread sleeping for {} seconds",
            name,
            duration.as_secs()
        );
        thread::sleep(duration);
    })
}

#[cfg(test)]
pub(crate) fn setup_test_env() {
    use std::path::Path;
    use std::sync::Once;

    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        if Path::new(".env").is_file() {
            dotenv::dotenv().expect("failed to initialize dotenv");
        }
    });
}
