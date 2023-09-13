use rocket::outcome::Outcome;

pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

impl BasicAuth {
    pub fn from_authorization_header(header: &str) -> Option<BasicAuth> {
        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.len() != 2 {
            return None;
        }

        if parts[0] != "Basic" {
            return None;
        }

        let decoded = base64::decode(parts[1]).ok()?;
        let decoded = String::from_utf8(decoded).ok()?;
        let parts: Vec<&str> = decoded.split(":").collect();
        if parts.len() != 2 {
            return None;
        }

        Some(BasicAuth {
            username: parts[0].to_string(),
            password: parts[1].to_string(),
        })
    }
}

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for BasicAuth {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let auth_header = request.headers().get_one("Authorization");
        if let Some(auth_header) = auth_header {
            if let Some(basic_auth) = BasicAuth::from_authorization_header(auth_header) {
                return Outcome::Success(basic_auth);
            }
        }

        Outcome::Failure((rocket::http::Status::Unauthorized, ()))
    }
}