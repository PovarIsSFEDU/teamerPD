use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{Message, SmtpTransport, Transport};
use crate::prelude::*;

const SMTP_SERVER: &str = "smtp-relay.sendinblue.com";
const SENDER: &str = "p_lukash@list.ru";
const SMTP_LOGIN: &str = "p_lukash@list.ru";

pub fn send_email_verification(email: String, link: String) {
    send(email, "Verification", link);
}

pub fn send_email_change(email: String, link: String) {
    send(email, "Change email", link);
}

pub fn send_recovery(email: String, link: String) {
    send(email, "Password recovery", link);
}

fn send<S: Into<String>>(email: String, subject: S, link: String) {
    println!("Email: {}", email);
    let email = Message::builder()
        .from(SENDER.parse().unwrap())
        .to(email.parse().unwrap())
        .subject(subject)
        .body(link)
        .unwrap();

    let creds = Credentials::new(SMTP_LOGIN.to_owned(), from_config("mailing_key"));

    let mailer = SmtpTransport::builder_dangerous(SMTP_SERVER)
        .port(587)
        .credentials(creds.clone())
        .authentication(vec![Mechanism::Plain])
        .build();
    let mailer = SmtpTransport::relay(SMTP_SERVER)
        .unwrap()
        .port(465)
        .credentials(creds.clone())
        .authentication(vec![Mechanism::Plain])
        .build();
    //mailer.test_connection().unwrap();
    let res = mailer.send(&email);
    println!("Success: {}", res.is_ok());
    let res = res.unwrap();
    println!("||| {:?} ||| {:?}", res.is_positive(), res.code());
    for d in res.message() {
        println!("||| {} ", d);
    }
}