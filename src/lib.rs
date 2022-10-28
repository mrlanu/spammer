use serde::Deserialize;
use std::time::Duration;
use ureq::Cookie;
use ureq::{Agent, AgentBuilder};

pub fn run(config: Config) -> Result<(), ureq::Error> {
    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();

    let login_resp: Nonce = agent.post(&format!("{}/api/v1/auth/login", config.server))
        .set("Content-Type", "application/json")
        .set("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.88 Safari/537.36")
        .set("origin", "https://ts20.x2.international.travian.com")
        .set("authorization", "Bearer undefined")
        .send_json(ureq::json!({"name": config.login, "password": config.pass, "w": "1920:1080", "mobileOptimizations": false}))?
        .into_json()?;

    let auth_resp = agent.post(&format!("{}/api/v1/auth/{}", config.server, login_resp.nonce))
        .set("Content-Type", "application/json")
        .set("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.88 Safari/537.36")
        .set("origin", "https://ts20.x2.international.travian.com")
        .set("authorization", "Bearer undefined")
        .call()?;

    let set_cookie = auth_resp
        .header("set-cookie")
        .expect("Error: no header set-cookie");

    let cookie = Cookie::parse(set_cookie).unwrap();

    let stat_resp = agent.get(&format!("{}/statistics/player/overview?page=1", config.server))
        .set("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9")
        .set("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.88 Safari/537.36")
        .set("referer", &config.server)
        .set("cookie", &format!("{}={}",cookie.name(), cookie.value()))
        .call()?
    .into_string()?;

    println!("{}", stat_resp);

    Ok(())
}
#[derive(Deserialize, Debug)]
struct Nonce {
    nonce: String,
}

pub struct Config {
    server: String,
    login: String,
    pass: String,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enought arguments");
        }
        Ok(Config {
            server: args[1].clone(),
            login: args[2].clone(),
            pass: args[3].clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
