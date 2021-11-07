use lettre::{SmtpClient, SmtpTransport, Transport};
use lettre::smtp::error::SmtpResult;
use lettre_email::EmailBuilder;

const SMTP_SERVER: &str = "smtp-relay.sendinblue.com:587";
const SENDER: &str = "example@example.com";

pub fn send_email_verification(email: String, link: String) {
    send(email, "Verification", link);
}

pub fn send_email_change(email: String, link: String) {
    send(email, "Change email", link);
}

pub fn send_recovery(email: String, link: String) {
    send(email, "Password recovery", link);
}

fn send<S: Into<String>>(email: String, subject: S, link: String) -> SmtpResult {
    let email = EmailBuilder::new()
        .to(email)
        .from(SENDER)
        .subject(subject)
        .text(link)
        .build()
        .unwrap()
        .into();

    let mailer = SmtpClient::new_simple(SMTP_SERVER).unwrap();
    let mut mailer = SmtpTransport::new(mailer);
    let result = mailer.send(email);
    if result.is_ok() {
        println!("Email sent")
    } else {
        println!("Error {:?}", result)
    }

    result
}