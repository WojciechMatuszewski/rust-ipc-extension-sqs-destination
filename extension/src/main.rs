use std;
mod client;
use actix_web::{
    error::{self},
    post, web, App, Error, HttpResponse, HttpServer,
};
use anyhow::{anyhow, Context};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base_url = std::env::var("AWS_LAMBDA_RUNTIME_API").unwrap();
    println!("Extension base URL: {}", base_url);

    let mut client = client::Client::new(base_url);
    let register_response = client.register().await?;
    dbg!(register_response);

    tokio::spawn(async move {
        println!("Waiting for the next event");
        while let Ok(control_flow) = client.next_event().await {
            if control_flow.is_break() {
                break;
            }
        }
    });

    println!("Starting web server");

    HttpServer::new(move || App::new().service(retrieve_payload))
        .bind(("127.0.0.1", 8080))
        .context("Failed to bind to a given port")?
        .run()
        .await?;

    return Ok(());
}

#[post("/")]
async fn retrieve_payload(req_body: String) -> Result<HttpResponse, Error> {
    println!("Starting the process of sending the payload to the queue");

    let queue_url = std::env::var("DESTINATION_QUEUE_URL").map_err(|e| {
        return error::ErrorInternalServerError(anyhow!(
            "Failed to read the DESTINATION_QUEUE_URL variable: {:?}",
            e
        ));
    })?;

    println!("I have the desiccation queue URL");

    let sqs_client = aws_sdk_sqs::Client::new(&aws_config::load_from_env().await);
    sqs_client
        .send_message()
        .queue_url(queue_url)
        .message_body(req_body)
        .send()
        .await
        .map_err(|e| {
            println!("Error while reaching out to SQS: {:?}", e);
            return error::ErrorInternalServerError(anyhow!(
                "Failed to send the message to SQS: {:?}",
                e
            ));
        })?;

    println!("Message send!");

    return Ok(HttpResponse::Ok().json(web::Json(json!({
        "Message": "success"
    }))));
}
