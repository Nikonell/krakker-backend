use apistos::{info::Info, spec::Spec, tag::Tag};

pub fn get_spec() -> Spec {
    Spec {
        info: Info {
            title: "Krakker API".to_string(),
            version: "1.0.0".to_string(),
            description: Some("API documentation for Krakker".to_string()),
            ..Default::default()
        },
        tags: vec![
            Tag {
                name: "Auth".to_string(),
                description: Some("Authentication operations".to_string()),
                ..Default::default()
            },
            Tag {
                name: "Users".to_string(),
                description: Some("User operations".to_string()),
                ..Default::default()
            },
            Tag {
                name: "Projects".to_string(),
                description: Some("Project operations".to_string()),
                ..Default::default()
            },
            Tag {
                name: "Notifications".to_string(),
                description: Some("Notification operations".to_string()),
                ..Default::default()
            }
        ],
        ..Default::default()
    }
}
