use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;
use log::error;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub token: String,
    pub id: String,
}

impl From<PathBuf> for User {
    fn from(user_path: PathBuf) -> Self {
        let userfile = OpenOptions::new()
            .read(true)
            .write(true)
            .open(user_path)
            .expect("Problem opening user info file");
        let userinfo = serde_json::from_reader(userfile).unwrap();
        serde_json::from_value(userinfo).expect("Invalid user json file")
    }
}

pub async fn handle_auth() -> Result<User, ()> {
    let mut email: String = "".to_string();

    print!("Log in to mojang\nEmail: ");

    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut email).unwrap();

    email = email.trim_end().to_string();

    let password: String = rpassword::prompt_password_stdout("Password: ").unwrap();

    let user = authenticate(email.as_str(), password.as_str()).await;
    if user.is_none() {
        error!("Not a valid user. Try again.");
        Err(())
    } else {
        Ok(user.unwrap())
    }
}

pub async fn authenticate(email: &str, password: &str) -> Option<User> {
    let request_client : reqwest::Client =  reqwest::Client::new();
    let payload = serde_json::json!(
    {
        "agent" : {
            "name": "Minecraft",
            "version" : 1
        },
        "username" : email,
        "password" : password
    });

    let resp = request_client.post(
            reqwest::Url::parse("https://authserver.mojang.com/authenticate")
            .unwrap()
            )
                .json(&payload)
                .send()
                .await;

    //let auth_data : serde_json::Value = serde_json::from_str(resp.text().await.unwrap().as_str()).unwrap();
    match resp {
        Ok(userinfo) => {
            let userinfo_json: serde_json::Value =
                serde_json::from_str(userinfo.text().await.unwrap().as_str()).expect("Invalid response data");
                //userinfo.into_json().expect("Error parsing auth json");

            let access_token = userinfo_json["accessToken"].clone();
            let username = userinfo_json["selectedProfile"]["name"].clone();
            let uuid = userinfo_json["selectedProfile"]["id"].clone();

            Some(User {
                name: username.as_str().expect("Error parsing json").to_string(),
                token: access_token
                    .as_str()
                    .expect("Error parsing json")
                    .to_string(),

                id: uuid.as_str().expect("Error getting uuid").to_string()
            })
        },
        //Err(ureq::Error::Status(code, _resp)) => {
        //    println!("Got status {}", code);

        //    handle_auth()
        //}
        Err(_) => None,
    }
 

    // send payload here
    //match ureq::post("https://authserver.mojang.com/authenticate").send_json(payload) {
    //    Ok(userinfo) => {
    //        let userinfo_json: serde_json::Value =
    //            userinfo.into_json().expect("Error parsing auth json");

    //        let access_token = userinfo_json["accessToken"].clone();
    //        let username = userinfo_json["selectedProfile"]["name"].clone();

    //        Some(User {
    //            name: username.as_str().expect("Error parsing json").to_string(),
    //            token: access_token
    //                .as_str()
    //                .expect("Error parsing json")
    //                .to_string(),
    //        })
    //    }
    //    Err(ureq::Error::Status(code, _resp)) => {
    //        println!("Got status {}", code);

    //        handle_auth()
    //    }
    //    Err(_) => handle_auth(),
    //}
}
