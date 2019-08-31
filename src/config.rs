use crate::ResponseType;
use std::collections::HashMap;

pub(crate) struct Config {
    pub(crate) default_response_type: ResponseType,
    pub(crate) response_types: HashMap<String, ResponseType>,
    pub(crate) base_path_behaviour: BasePathBehaviour,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BasePathBehaviour {
    RemountAndInclude,
    Include,
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
