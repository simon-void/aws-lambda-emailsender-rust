use std::collections::HashMap;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_sesv2::model::{Body, Content, Destination, EmailContent, Message};
use aws_sdk_sesv2::{Client, Region};
use log::info;
use serde::{Serialize};
use structopt::StructOpt;


const VERIFIED_EMAIL_KEY: &str = "verified_SES_email";
const RECEIVER_EMAIL_BY_NAME_KEY: &str = "receiver_email_by_name_csv";
const NOT_FOUND: &str = "not found";

#[derive(Debug, StructOpt)]
struct Request {
    pub subject: Option<String>,
    pub message: Option<String>,
    pub senderEmail: Option<String>,
    pub receiverName: Option<String>,
}

struct MessageData {
    pub subject: String,
    pub body: String,
    pub sender_email: Option<String>,
    pub receiver_email: String,
}

// class Request {
//   var subject: String? = null
//   var message: String? = null
//   var senderEmail: String? = null
//   var receiverName: String? = null
// }

#[derive(Debug, Serialize)]
struct SuccessResponse {
    pub wasSuccessful: bool,
    pub errorMessage: String,
}

impl SuccessResponse {
    fn new() -> Self {
        SuccessResponse {
            wasSuccessful: true,
            errorMessage: "".into(),
        }
    }
}

#[derive(Debug, Serialize)]
struct FailureResponse {
    pub wasSuccessful: bool,
    pub errorMessage: String,
}

impl FailureResponse {
    fn new(errorMessage: String) -> Self {
        FailureResponse {
            wasSuccessful: false,
            errorMessage,
        }
    }
}

// Implement Display for the Failure response so that we can then implement Error.
impl std::fmt::Display for FailureResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.errorMessage)
    }
}

// Implement Error for the FailureResponse so that we can `?` (try) the Response
// returned by `lambda_runtime::run(func).await` in `fn main`.
impl std::error::Error for FailureResponse {}

type Response = Result<SuccessResponse, FailureResponse>;

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    let request = Request::from_args();

    handler(request).await?

    Ok(())
}

async fn handler(req: Request) -> Response {
    info!("handling a request...");
    let verified_ses_from_email: String = std::env::var(VERIFIED_EMAIL_KEY).expect(
        format!("A {VERIFIED_EMAIL_KEY} must be set in this app's Lambda environment variables.")
            .as_str(),
    );
    let receiver_email_by_name: String = std::env::var(RECEIVER_EMAIL_BY_NAME_KEY)
        .expect(format!("A {RECEIVER_EMAIL_BY_NAME_KEY} must be set in this app's Lambda environment variables.").as_str().clone());
    let provided_to_email_by_name: HashMap<&str, &str> = split_to_map(receiver_email_by_name.as_ref());

    let message: MessageData = extract_message(req, provided_to_email_by_name)?;

    let client = {
        let region_provider =
            RegionProviderChain::first_try(Region::new("eu-west-1")).or_default_provider();

        let shared_config = aws_config::from_env().region(region_provider).load().await;
        Client::new(&shared_config)
    };

    send_email(&client, message, verified_ses_from_email).await
}

async fn send_email(
    ses_client: &Client,
    message: MessageData,
    verified_ses_from_email: String,
) -> Result<SuccessResponse, FailureResponse> {
    let dest = Destination::builder()
        .to_addresses(message.receiver_email)
        .build();
    let subject_content = Content::builder()
        .data(format!(
            "senderEmail: {}\n\nmessage: {}",
            message.sender_email.unwrap_or("not provided".into()),
            message.body
        ))
        .charset("UTF-8")
        .build();
    let body_content = Content::builder()
        .data(message.body)
        .charset("UTF-8")
        .build();
    let body = Body::builder().text(body_content).build();

    let msg = Message::builder()
        .subject(subject_content)
        .body(body)
        .build();

    let email_content = EmailContent::builder().simple(msg).build();

    ses_client
        .send_email()
        .from_email_address(verified_ses_from_email.as_str())
        .destination(dest)
        .content(email_content)
        .send()
        .await
        .map_err(|err|FailureResponse::new(err.to_string()))?;

    Ok(SuccessResponse::new())
}

fn split_to_map(receiver_email_by_name_list: &str) -> HashMap<&str, &str> {
    // split receiver_email_by_name_list first by '=' and then by '=' because that's the expected format
    // e.g. 'person1=guy@gmail.com, dojo=dojo.owner@yahoo.com'
    receiver_email_by_name_list
        .split(',')
        .filter_map(|key_and_name| {
            let mut key_and_name_iter = key_and_name.split('=');
            match key_and_name_iter.next() {
                Some(key) => match key_and_name_iter.next() {
                    Some(name) => {
                        let trimmed_key = key.trim();
                        let trimmed_name = name.trim();
                        if !trimmed_key.is_empty() && !trimmed_name.is_empty() {
                            Some((trimmed_key, trimmed_name))
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                _ => None,
            }
        })
        .collect()
}

fn extract_message(
    request: Request,
    email_by_provided_name: HashMap<&str, &str>,
) -> Result<MessageData, FailureResponse> {
    let subject = request.subject.ok_or(FailureResponse::new(
        "subject expected but not found".into(),
    ))?;
    let body = request.message.ok_or(FailureResponse::new(
        "message expected but not found".into(),
    ))?;
    let receiver_name = request.receiverName.ok_or(FailureResponse::new(
        "receiverName expected but not found".into(),
    ))?;
    let receiver_email: String = email_by_provided_name
        .get(receiver_name.as_str())
        .map(|&email| email.into())
        .ok_or(FailureResponse::new(format!(
            "no email for receiverName {receiver_name}"
        )))?;
    Ok(MessageData {
        subject,
        body,
        sender_email: request.senderEmail,
        receiver_email,
    })
}
