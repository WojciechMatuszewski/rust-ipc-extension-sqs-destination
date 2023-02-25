use std::ops::ControlFlow;

use anyhow::Context;
use serde::Deserialize;
use serde_json::json;

const LAMBDA_EXTENSION_IDENTIFIER_HEADER: &str = "Lambda-Extension-Identifier";
const LAMBDA_EXTENSION_NAME_HEADER: &str = "Lambda-Extension-Name";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    pub function_name: String,
    pub function_version: String,
    pub handler: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvokeResponse {
    pub deadline_ms: u64,
    pub request_id: String,
    pub invoked_function_arn: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShutdownResponse {
    pub shutdown_reason: String,
    pub deadline_ms: u64,
}

/// All possible next event responses.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE", tag = "eventType")]
pub enum NextEventResponse {
    Invoke(InvokeResponse),
    Shutdown(ShutdownResponse),
}

pub struct Client {
    base_url: String,
    extension_id: Option<String>,
}

impl Client {
    pub fn new(base_url: String) -> Self {
        let computed_url = format!("http://{}/2020-01-01/extension", base_url);
        println!("Extension will make requests to the URL: {}", computed_url);

        Self {
            base_url: computed_url,
            extension_id: None,
        }
    }

    pub async fn register(&mut self) -> anyhow::Result<RegisterResponse> {
        let url = format!("{}/register", self.base_url);
        let client = reqwest::Client::new();

        println!(
            "Registering the extension. Making a request to URL: {}",
            url
        );

        let response = client
            .post(url)
            .json(&json!({"events": ["INVOKE", "SHUTDOWN"]}))
            .header(LAMBDA_EXTENSION_NAME_HEADER, "extension")
            .send()
            .await
            .context("Failed to send the request to register the extension")?;

        let response_status = response.status();
        if response_status != reqwest::StatusCode::OK {
            let error_message = format!(
                "request failed with statusCode: {}, text: {}",
                response_status.as_str(),
                response.text().await?
            );
            panic!("{}", error_message);
        }

        let extension_id = response
            .headers()
            .get(LAMBDA_EXTENSION_IDENTIFIER_HEADER)
            /*
                First use the `.and_then` operator.
                This operator allows us to chain operations on the `Option` values.
                If the `Option` is None, the transformation function will not be invoked.
            */
            .and_then(|h| h.to_str().ok())
            /*
                And now we map the Option<U> to Option<T>
            */
            .map(|h| h.to_string());

        self.extension_id = extension_id;
        let response = response.json::<RegisterResponse>().await?;
        return Ok(response);
    }

    /**
     * https://github.com/getsentry/relay/tree/master/relay-aws-extension
     */
    pub async fn next_event(&self) -> anyhow::Result<ControlFlow<()>, reqwest::Error> {
        let extension_id = match &self.extension_id {
            Some(value) => value,
            None => panic!("Please register the extension first"),
        };

        let url = format!("{}/event/next", self.base_url);
        println!("Waiting for the next event. Blocking on the URL: {}", url);

        let client = reqwest::Client::new();

        let response = client
            .request(reqwest::Method::GET, url)
            .header(LAMBDA_EXTENSION_IDENTIFIER_HEADER, extension_id)
            .send()
            .await?
            .json::<NextEventResponse>()
            .await?;

        match response {
            NextEventResponse::Invoke(response) => {
                dbg!(response);
                return Ok(ControlFlow::Continue(()));
            }
            NextEventResponse::Shutdown(response) => {
                /*
                   Send the response to AWS Lambda destination?
                */
                dbg!(response);
                return Ok(ControlFlow::Break(()));
            }
        }
    }
}
