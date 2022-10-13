# aws-lambda-emailsender-rust

AWS Lambda in Rust based on sources
- AWS example of using SES in a Rust lambda: https://github.com/awslabs/aws-sdk-rust/blob/main/examples/ses/src/bin/send-email.rs
- AWS SDK for Rust lambda with S3 example: https://docs.aws.amazon.com/sdk-for-rust/latest/dg/lambda.html
- Developer Preview(!!) of SES Rust crate v2: [aws-sdk-sesv2](https://crates.io/crates/aws-sdk-sesv2)

This Lambda allows to send emails to receivers only known by a token to the sender.
This is for example useful to allow a (potentially Cross-Origin) website to
send emails to the website's maintainer via a POST-request to the API Gateway
without leaking their email address to people tech-savvy enough to inspect the
sourcecode of the website.

### Environment Variables that must be set:

+ `verified_SES_email`: an email like `example@gmail.com`
+ `receiver_email_by_name_csv`: a comma separated list of {name}={email} like `person1=guy@gmail.com, dojo=dojo.owner@yahoo.com`

### Parameters that are assumed:
+ `subject`: String - the subject of the email to send
+ `message`: String - the message of the email to send
+ `senderEmail`: String? - the (optional) email address of the sender
+ `receiverName`: String - must be a key in the map defined by `receiver_email_by_name_csv`. The email will be send to the email address linked to that name.