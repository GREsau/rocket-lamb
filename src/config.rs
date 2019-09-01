use std::collections::HashMap;

pub(crate) struct Config {
    pub(crate) default_response_type: ResponseType,
    pub(crate) response_types: HashMap<String, ResponseType>,
    pub(crate) base_path_behaviour: BasePathBehaviour,
}

/// Determines how to encode response content. The default is `Text`.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ResponseType {
    /// Encodes response content as a UTF-8 string.
    Text,
    /// Encodes response content as Base64.
    Binary,
}

/// Determines whether the API Gateway base path is included in the URL processed by Rocket.
/// The default is `RemountAndInclude`.
#[derive(Debug, PartialEq, Eq)]
pub enum BasePathBehaviour {
    /// Includes the base bath in the URL. The first request received will be used to determine
    /// the base path, and all mounted routes will be cloned and re-mounted at the base path.
    RemountAndInclude,
    /// Includes the base bath in the URL. You must ensure that the `Rocket`'s routes have been
    /// mounted at the expected base path.
    Include,
    /// Excludes the base bath from the URL. The URL processed by Rocket may not match the full
    /// path of the original client, which may cause absolute URLs in responses (e.g. in the
    /// `Location` response header for redirects) to not behave as expected.
    Exclude,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            default_response_type: ResponseType::Text,
            response_types: HashMap::new(),
            base_path_behaviour: BasePathBehaviour::RemountAndInclude,
        }
    }
}
