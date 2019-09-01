use http::header::HOST;
use lambda_http::request::RequestContext;
use lambda_http::{Request, RequestExt as _};

pub(crate) trait RequestExt {
    fn full_path(&self) -> String;

    fn base_path(&self) -> String;

    fn api_path(&self) -> &str;
}

impl RequestExt for Request {
    fn full_path(&self) -> String {
        if self.request_context().is_alb() || !is_default_api_gateway_url(self) {
            self.uri().path().to_owned()
        } else {
            let mut path = self.base_path();
            path.push_str(self.uri().path());
            path
        }
    }

    fn base_path(&self) -> String {
        match self.request_context() {
            RequestContext::ApiGateway {
                stage,
                resource_path,
                ..
            } => {
                if is_default_api_gateway_url(self) {
                    format!("/{}", stage)
                } else {
                    let resource_path = populate_resource_path(self, resource_path);
                    let full_path = self.uri().path();
                    let resource_path_index =
                        full_path.rfind(&resource_path).unwrap_or_else(|| {
                            panic!(
                                "Could not find segment '{}' in path '{}'.",
                                resource_path, full_path
                            )
                        });
                    full_path[..resource_path_index].to_owned()
                }
            }
            RequestContext::Alb { .. } => String::new(),
        }
    }

    fn api_path(&self) -> &str {
        if self.request_context().is_alb() || is_default_api_gateway_url(self) {
            self.uri().path()
        } else {
            &self.uri().path()[self.base_path().len()..]
        }
    }
}

fn is_default_api_gateway_url(req: &Request) -> bool {
    req.headers()
        .get(HOST)
        .and_then(|h| h.to_str().ok())
        .map(|h| h.ends_with(".amazonaws.com") && h.contains(".execute-api."))
        .unwrap_or(false)
}

fn populate_resource_path(req: &Request, resource_path: String) -> String {
    let path_parameters = req.path_parameters();
    resource_path
        .split('/')
        .map(|segment| {
            if segment.starts_with('{') {
                let end = if segment.ends_with("+}") { 2 } else { 1 };
                let param = &segment[1..segment.len() - end];
                path_parameters
                    .get(param)
                    .unwrap_or_else(|| panic!("Could not find path parameter '{}'.", param))
            } else {
                segment
            }
        })
        .collect::<Vec<&str>>()
        .join("/")
}
