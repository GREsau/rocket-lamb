use crate::ResponseType;
use std::collections::HashMap;

pub(crate) struct Config {
    pub(crate) default_response_type: ResponseType,
    pub(crate) response_types: HashMap<String, ResponseType>,
    pub(crate) include_api_gateway_base_path: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            default_response_type: ResponseType::Text,
            response_types: HashMap::new(),
            include_api_gateway_base_path: true,
        }
    }
}
