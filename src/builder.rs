use crate::config::{BasePathBehaviour, Config};
use crate::handler::{LazyClient, RocketHandler};
use lambda_http::lambda;
use rocket::Rocket;

/// Used to determine how to encode response content. The default is `Text`.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ResponseType {
    /// Send response content to API Gateway as a UTF-8 string.
    Text,
    /// Send response content to API Gateway Base64-encoded.
    Binary,
}

/// A builder to create and configure a [RocketHandler](RocketHandler).
pub struct RocketHandlerBuilder {
    rocket: Rocket,
    config: Config,
}

impl RocketHandlerBuilder {
    /// Create a new `RocketHandlerBuilder`. Alternatively, you can use [rocket.lambda()](crate::RocketExt::lambda).
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::RocketHandlerBuilder;
    ///
    /// let builder = RocketHandlerBuilder::new(rocket::ignite());
    /// ```
    pub fn new(rocket: rocket::Rocket) -> RocketHandlerBuilder {
        RocketHandlerBuilder {
            rocket,
            config: Config::default(),
        }
    }

    /// Creates a new `RocketHandler` from an instance of `Rocket`, which can be passed to the [lambda_http::lambda!](lambda_http::lambda) macro.
    ///
    /// Alternatively, you can use the [launch()](RocketHandlerBuilder::launch) method.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rocket_lamb::RocketExt;
    /// use lambda_http::lambda;
    ///
    /// let handler = rocket::ignite().lambda().into_handler();
    /// lambda!(handler);
    /// ```
    pub fn into_handler(self) -> RocketHandler {
        RocketHandler {
            client: LazyClient::Uninitialized(self.rocket),
            config: self.config,
        }
    }

    /// Starts handling Lambda events by polling for events using Lambda's Runtime APIs.
    ///
    /// This function does not return, as it will loop forever (unless it panics).
    ///
    /// # Panics
    ///
    /// This panics if the required Lambda runtime environment variables are not set, or if the `Rocket` used to create the builder was misconfigured.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rocket_lamb::RocketExt;
    /// use lambda_http::lambda;
    ///
    /// rocket::ignite().lambda().launch();
    /// ```
    pub fn launch(self) -> ! {
        lambda!(self.into_handler());
        unreachable!("lambda! should loop forever (or panic)")
    }

    /// Gets the default `ResponseType`, which is used for any responses that have not had their Content-Type overriden with [response_type](RocketHandlerBuilder::response_type).
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::{RocketExt, ResponseType};
    ///
    /// let builder = rocket::ignite().lambda();
    /// assert_eq!(builder.get_default_response_type(), ResponseType::Text);
    /// assert_eq!(builder.get_response_type("text/plain"), ResponseType::Text);
    /// ```
    pub fn get_default_response_type(&self) -> ResponseType {
        self.config.default_response_type
    }

    /// Sets the default `ResponseType`, which is used for any responses that have not had their Content-Type overriden with [response_type](RocketHandlerBuilder::response_type).
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::{RocketExt, ResponseType};
    ///
    /// let builder = rocket::ignite()
    ///     .lambda()
    ///     .default_response_type(ResponseType::Binary);
    /// assert_eq!(builder.get_default_response_type(), ResponseType::Binary);
    /// assert_eq!(builder.get_response_type("text/plain"), ResponseType::Binary);
    /// ```
    pub fn default_response_type(mut self, response_type: ResponseType) -> Self {
        self.config.default_response_type = response_type;
        self
    }

    /// Gets the configured `ResponseType` for responses with the given Content-Type header.
    ///
    /// `content_type` values are treated case-insensitively.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::{RocketExt, ResponseType};
    ///
    /// let builder = rocket::ignite()
    ///     .lambda()
    ///     .response_type("TEXT/PLAIN", ResponseType::Binary);
    /// assert_eq!(builder.get_response_type("text/plain"), ResponseType::Binary);
    /// assert_eq!(builder.get_response_type("application/json"), ResponseType::Text);
    /// ```
    pub fn get_response_type(&self, content_type: &str) -> ResponseType {
        self.config
            .response_types
            .get(&content_type.to_lowercase())
            .copied()
            .unwrap_or(self.config.default_response_type)
    }

    /// Sets the `ResponseType` for responses with the given Content-Type header.
    ///
    /// `content_type` values are treated case-insensitively.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::{RocketExt, ResponseType};
    ///
    /// let builder = rocket::ignite()
    ///     .lambda()
    ///     .response_type("TEXT/PLAIN", ResponseType::Binary);
    /// assert_eq!(builder.get_response_type("text/plain"), ResponseType::Binary);
    /// assert_eq!(builder.get_response_type("application/json"), ResponseType::Text);
    /// ```
    pub fn response_type(mut self, content_type: &str, response_type: ResponseType) -> Self {
        self.config
            .response_types
            .insert(content_type.to_lowercase(), response_type);
        self
    }

    /// TODO update documentation
    ///
    /// Sets whether or not the handler should determine the API Gateway base path and prepend it to the path of request URLs.
    ///
    /// By default, this setting is `true`.
    ///
    /// When using the default API Gateway URL `1234567890.execute-api.region.amazonaws.com/{stage}/`, then the base path would
    /// be `/{stage}`. If this setting is set to `true` (the default), then all mounted routes will be made available under
    /// `/{stage}`, and all incoming requests to Rocket will have `/{stage}` at the beginning of the URL path. This is necessary
    /// to make absolute URLs in responses (e.g. in the `Location` response header for redirects) function correctly when
    /// hosting the server using the default API Gateway URL.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::{BasePathBehaviour, RocketExt};
    ///
    /// let builder = rocket::ignite()
    ///     .lambda()
    ///     .base_path_behaviour(BasePathBehaviour::Exclude);
    /// ```
    pub fn base_path_behaviour(mut self, setting: BasePathBehaviour) -> Self {
        self.config.base_path_behaviour = setting;
        self
    }
}
