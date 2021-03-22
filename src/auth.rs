use std::io::{self, Write};

use crate::sml::User;

pub fn handle_auth() -> Option<User> {
    let mut email: String = "".to_string();

    print!("Log in to mojang\nEmail: ");

    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut email).unwrap();

    email = email.trim_end().to_string();

    let password: String = rpassword::prompt_password_stdout("Password: ").unwrap();

    let user = authorize(email.as_str(), password.as_str());

    if user.is_none() {
        handle_auth()
    }else {
        Some(user.unwrap())
    }

}

pub fn authorize(email: &str, password: &str) -> Option<User> {
    let payload = serde_json::json!(
    {
        "agent" : {
            "name": "Minecraft",
            "version" : 1
        },
        "username" : email,
        "password" : password
    });

    // send payload here
    match ureq::post("https://authserver.mojang.com/authenticate").send_json(payload) {
        Ok(userinfo) => {
            let userinfo_json: serde_json::Value =
                userinfo.into_json().expect("Error parsing auth json");

            let access_token = userinfo_json["accessToken"].clone();
            let username = userinfo_json["selectedProfile"]["name"].clone();

            Some(User {
                name: username.as_str().expect("Error parsing json").to_string(),
                token: access_token
                    .as_str()
                    .expect("Error parsing json")
                    .to_string(),
            })
        },
        Err(ureq::Error::Status(code, resp)) => {

            let err_json : serde_json::Value = resp.into_json().unwrap();
            println!("Got status {}", code);

            if code == 403 {
                return handle_auth();
            } else {
                return handle_auth();
            }
        }
        Err(_) => {
            return handle_auth();
        }
    }
}
