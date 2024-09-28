use crate::mailer::mailer::Mailer;

#[derive(Clone)]
pub struct AppData {
    pub mailer: Mailer,
}
