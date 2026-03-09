use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::env;

pub struct MailService {
    transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
    from_email: String,
    stub: bool,
}

impl MailService {
    pub fn new() -> Self {
        let stub = env::var("SMTP_STUB").map(|v| v == "true").unwrap_or(false);
        let from_email =
            env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@example.com".to_string());

        if stub {
            return Self {
                transport: None,
                from_email,
                stub: true,
            };
        }

        let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");
        let smtp_port = env::var("SMTP_PORT")
            .expect("SMTP_PORT must be set")
            .parse::<u16>()
            .expect("SMTP_PORT must be a number");
        let smtp_user = env::var("SMTP_USER").ok();
        let smtp_password = env::var("SMTP_PASSWORD").ok();

        let mut mailer_builder = if smtp_port == 465 {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)
                .expect("Failed to create SMTP relay")
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)
                .expect("Failed to create SMTP relay")
        };

        let credentials = smtp_user.zip(smtp_password).filter(|(u, _)| !u.is_empty());
        if let Some((user, pass)) = credentials {
            mailer_builder = mailer_builder.credentials(Credentials::new(user, pass));
        }

        let transport = mailer_builder.port(smtp_port).build();

        Self {
            transport: Some(transport),
            from_email,
            stub: false,
        }
    }

    pub async fn send_email(&self, to: &str, subject: &str, body: String) -> Result<(), String> {
        if self.stub {
            tracing::info!(to = %to, subject = %subject, body = %body, "STUB EMAIL sent");
            return Ok(());
        }

        let email = Message::builder()
            .from(
                self.from_email
                    .parse()
                    .map_err(|e| format!("Invalid from email: {}", e))?,
            )
            .to(to.parse().map_err(|e| format!("Invalid to email: {}", e))?)
            .subject(subject)
            .body(body)
            .map_err(|e| format!("Failed to build email: {}", e))?;

        if let Some(transport) = &self.transport {
            transport
                .send(email)
                .await
                .map_err(|e| format!("Failed to send email: {}", e))?;
        }

        Ok(())
    }
}
