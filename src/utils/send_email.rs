use std::env;

use axum::http::StatusCode;
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use maud::{html, Markup};

pub async fn send_mail(name: &String, email: &String, otp: &i32) -> Result<bool, StatusCode> {
    let email_content: Markup = html! {
        head {
            title { "OTP Verification - Clicon.io" }
            style type="text/css" {
                "body { font-family: Arial, Helvetica, sans-serif; text-align: center; padding: 20px; background-color: #f4f4f4; }"
                ".container { max-width: 500px; background: #fff; padding: 20px; border-radius: 8px; box-shadow: 0px 4px 10px rgba(0,0,0,0.1); text-align: left; }"
                "h2 { color: #333; margin-bottom: 15px; }"
                "p { font-size: 16px; color: #555; line-height: 1.6; margin-bottom: 10px; }"
                ".otp-container { font-size: 24px; font-weight: bold; color: #d9534f; background: #f8d7da; padding: 15px; border-radius: 5px; display: inline-block; margin: 15px 0; }"
                ".footer { font-size: 12px; color: #777; margin-top: 20px; }"
            }
        }
        body {
            div class="container" style="padding: 20px;" {
                h2 { "OTP Verification" }
                p { "Dear " (name) "," }
                p { "Use the OTP below to verify your email:" }
                div class="otp-container" { (otp) }
                p { "This OTP is valid for 5 minutes. Do not share it with anyone." }
                p class="footer" { "If you didn’t request this, you can ignore this email." }
            }
        }
    };

    let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME is not set in .env");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD is not set in .env");

    let email = Message::builder()
        .from("Clicon.io <no-reply@clicon.io>".parse().unwrap())
        .reply_to("Support <support@clicon.io>".parse().unwrap())
        .to(format!("{} <{}>", name, email).parse().unwrap())
        .subject("Your OTP Code - Clicon.io")
        .header(ContentType::TEXT_HTML)
        .body(email_content.into_string())
        .unwrap();

    let creds = Credentials::new(smtp_username.to_owned(), smtp_password.to_owned());

    // Open a remote connection to gmail
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
            .unwrap()
            .credentials(creds)
            .build();

    // Send the email
    match mailer.send(email).await {
        Ok(_) => Ok(true),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
