use serde::Serialize;
use serde_json::{Map, Value};

/// Args part of the capabilities.
#[derive(Serialize)]
pub enum Args {
    /// Firefox is run as normal, minus any visible UI components visible.
    ///
    /// See [Headless mode](https://developer.mozilla.org/en-US/docs/Mozilla/Firefox/Headless_mode) for more info.
    ///
    /// # Note
    /// Serialized as `-headless` for the capabilities format.
    #[serde(rename = "-headless")]
    Headless,
}

/// Used to control the behaviour of Firefox.
///
/// See [firefoxOptions](https://developer.mozilla.org/en-US/docs/Web/WebDriver/Capabilities/firefoxOptions) for more info.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub accept_insecure_certs: bool,
    pub application_cache_enabled: bool,
    pub args: Vec<Args>,
    pub use_automation_extension: bool,
}

impl From<Capabilities> for Map<String, Value> {
    fn from(cap: Capabilities) -> Self {
        (&cap).into()
    }
}

impl From<&Capabilities> for Map<String, Value> {
    fn from(cap: &Capabilities) -> Self {
        let mut map = Map::with_capacity(1);

        map.insert(
            "moz:firefoxOptions".to_string(),
            serde_json::to_value(cap).expect("Gecko Capabilities"),
        );
        map
    }
}
