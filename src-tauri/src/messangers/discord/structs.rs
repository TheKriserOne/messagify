use serde::Deserialize;


#[derive(Deserialize)]
pub struct Profile {
    // accent_color: Option<String>,
    // authenticator_types: Vec<String>,
    // avatar: Option<String>,
    // avatar_decoration_data: Option<String>,
    // banner: Option<String>,
    // banner_color: Option<String>,
    // bio: String,
    // clan: Option<String>,
    // discriminator: String,
    // email: String,
    // flags: i32,
    // global_name: String,
    pub id: String,
    // linked_users: Vec<String>,
    // locale: String,
    // mfa_enabled: bool,
    // nsfw_allowed: bool,
    // phone: Option<String>,
    // premium_type: i32,
    // public_flags: i32,
    pub username: String,
    // verified: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct User {
    pub avatar: Option<String>,
    // avatar_decoration_data: Option<String>,
    // clan: Option<String>,
    // discriminator: String,
    pub id: String,
    pub username: String,
}
