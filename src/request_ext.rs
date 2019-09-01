use http::header::HOST;
use lambda_http::request::RequestContext;
use lambda_http::{Request, RequestExt as _};

pub(crate) trait RequestExt {
    fn base_path(&self) -> String;

    fn api_path(&self) -> &str;

    fn full_path(&self) -> String {
        let mut path = self.base_path();
        path.push_str(self.api_path());
        path
    }
}

impl RequestExt for Request {
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
                    match full_path.find(&resource_path) {
                        Some(i) => full_path[..i].to_owned(),
                        None => panic!(
                            "Could not find segment '{}' in path '{}'.",
                            resource_path, full_path
                        ),
                    }
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
        .unwrap_or_default()
        .ends_with(".amazonaws.com")
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
                    .expect("Path parameters should match resource path")
            } else {
                segment
            }
        })
        .collect::<Vec<&str>>()
        .join("/")
}
