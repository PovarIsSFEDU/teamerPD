use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::header::ContentType;
use crate::prelude::*;

const SMTP_SERVER: &str = "smtp-relay.sendinblue.com";
const SENDER: &str = "Teamer <p_lukash@list.ru>";
const SMTP_LOGIN: &str = "p_lukash@list.ru";

pub fn send_email_verification(email: String, link: String) -> Result<(), ()> {
    send(&email, "Verification", link)
}

pub fn send_email_change(email: String, link: String) -> Result<(), ()> {
    send(&email, "Change email", link)
}

pub fn send_recovery(email: String, link: String) -> Result<(), ()> {
    send(&email, "Password recovery", link)
}

pub fn send<S: Into<String>>(email: &String, subject: S, link: String) -> Result<(), ()> {
    let email = Message::builder()
        .from(SENDER.parse().unwrap())
        .to(email.parse().unwrap())
        .subject(subject)
        .header(ContentType::parse("text/html; charset=utf-8").unwrap())
        .body(format!("<a href=\"{}\">Click here to verify</a>", link))
        .unwrap();

    let creds = Credentials::new(SMTP_LOGIN.to_owned(), from_config("mailing_key"));

    let mailer = SmtpTransport::builder_dangerous(SMTP_SERVER)
        .port(587)
        .credentials(creds)
        .authentication(vec![Mechanism::Plain])
        .build();

    mailer
        .send(&email)
        .map_both(|_| (), |e| { println!("Error sending email: {:?}", e); () })
}