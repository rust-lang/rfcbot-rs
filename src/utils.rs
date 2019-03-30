#[cfg(test)]
pub(crate) fn setup_test_env() {
    use std::sync::Once;
    use std::path::Path;

    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        if Path::new(".env").is_file() {
            dotenv::dotenv().expect("failed to initialize dotenv");
        }
    });
}
