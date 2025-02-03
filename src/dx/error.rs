/// Macro shorthand for checking results
#[macro_export]
macro_rules! hr_bail {
    ($hr:expr, $($arg:tt)*) => {
        if winapi::shared::winerror::FAILED($hr) {
            anyhow::bail!($($arg)*);
        }
    };
}
