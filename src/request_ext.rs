use http::header::HOST;
use lambda_http::request::RequestContext;
use lambda_http::{Request, RequestExt as _};

pub(crate) trait RequestExt {
    fn base_path(&self) -> String;

    fn resource_path(&self) -> &str;

    fn full_path(&self) -> String {
        let mut path = self.base_path();
        path.push_str(self.resource_path());
        path
    }
}

impl RequestExt for Request {
    fn base_path(&self) -> String {
        match self.request_context() {
            RequestContext::ApiGateway { stage, .. } => {
                let host = self
                    .headers()
                    .get(HOST)
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or_default();
                if host.ends_with(".amazonaws.com") {
                    format!("/{}", stage)
                } else {
                    // TODO custom domain - check whether it includes base path
                    String::new()
                }
            }
            RequestContext::Alb { .. } => String::new(),
        }
    }

    fn resource_path(&self) -> &str {
        // TODO this may include the custom domain base path
        self.uri().path()
    }
}
