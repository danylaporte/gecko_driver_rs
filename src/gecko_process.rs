use std::process::Child;

/// Represent a GeckoProcess. This process is automatically killed on drop.
pub struct GeckoProcess(pub(crate) Child);

impl Drop for GeckoProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
    }
}
