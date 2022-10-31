use config::Config;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::thread;
use std::time::Duration;
use ureq::Agent;
use ureq::Cookie;

pub struct Messanger<'a> {
    config: AppConf,
    agent: Agent,
    cookie: Option<Cookie<'a>>,
    players: Vec<String>,
}
impl<'a> Messanger<'a> {
    pub fn build() -> Self {
        let conf = Config::builder()
            .add_source(config::File::with_name("Settings"))
            .add_source(config::Environment::with_prefix("APP"))
            .build()
            .unwrap();

        let config = conf.try_deserialize::<AppConf>().unwrap();

        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(10))
            .timeout_write(Duration::from_secs(10))
            .build();
        Self {
            config,
            agent,
            cookie: None,
            players: Vec::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), ureq::Error> {
        // println!("Conf: {:?}", self.config);

        self.login()?;

        let pages_amount = self.get_pages_amount();
        println!("Pages amount: {}", pages_amount);

        self.get_all_players(pages_amount);
        self.players.iter().for_each(|name| {
            println!("Sending the message to {}", name);
            self.send_message(&name)
                .expect("Error while sending a message");
            println!("Sent.");
            thread::sleep(Duration::from_secs(self.config.delay));
        });

        Ok(())
    }

    fn login(&mut self) -> Result<(), ureq::Error> {
        let login_resp: Nonce = self.agent.post(&format!("{}/api/v1/auth/login", self.config.server))
        .set("Content-Type", "application/json")
        .set("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.88 Safari/537.36")
        .set("origin", "https://ts20.x2.international.travian.com")
        .set("authorization", "Bearer undefined")
        .send_json(ureq::json!({"name": self.config.login, "password": self.config.pass, "w": "1920:1080", "mobileOptimizations": false}))?
        .into_json()?;

        let auth_resp = self.agent.post(&format!("{}/api/v1/auth/{}", self.config.server, login_resp.nonce))
        .set("Content-Type", "application/json")
        .set("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.88 Safari/537.36")
        .set("origin", "https://ts20.x2.international.travian.com")
        .set("authorization", "Bearer undefined")
        .call()?;

        let cookie_header = auth_resp
            .header("set-cookie")
            .expect("Error: no header set-cookie")
            .to_string();

        let cookie = Cookie::parse(cookie_header).unwrap();
        self.cookie = Some(cookie);
        Ok(())
    }

    fn get_pages_amount(&self) -> i32 {
        let stat_resp = self.get_statistic_page_by_number(1).unwrap();
        let document = Html::parse_document(&stat_resp);
        let select_paginator = Selector::parse("div.paginator").unwrap();
        let paginator = document.select(&select_paginator).next().unwrap();
        let a_number = Selector::parse("a.number").unwrap();

        let mut max_page = 0;

        for el in paginator.select(&a_number) {
            max_page = el.text().collect::<Vec<_>>()[0]
                .to_string()
                .parse::<i32>()
                .unwrap();
        }
        max_page
    }

    fn get_all_players(&mut self, pages_amount: i32) {
        for number in 1..=pages_amount {
            println!("Parsing page number - {}", number);
            let stat_resp = self.get_statistic_page_by_number(number).unwrap();
            let document = Html::parse_document(&stat_resp);

            let pla_class = Selector::parse(".pla ").unwrap();
            let a = Selector::parse("a").unwrap();

            for el in document.select(&pla_class).skip(1) {
                let names = el.select(&a).next().unwrap().text().collect::<Vec<_>>();
                let name = names.get(0).unwrap();
                self.players.push(name.to_string());
            }
            thread::sleep(Duration::from_millis(300));
        }
        println!("Added players {}", self.players.len());
        println!("List of players: {:?}", self.players);
        println!("Done.");
    }

    fn get_statistic_page_by_number(&self, number: i32) -> Result<String, ureq::Error> {
        let result = self.agent.get(&format!("{}/statistics/player/overview?page={}", self.config.server, number))
        .set("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9")
        .set("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.88 Safari/537.36")
        .set("referer", &self.config.server)
        .set("cookie", &format!("{}={}", self.cookie.as_ref().unwrap().name(), self.cookie.as_ref().unwrap().value()))
        .call()?
        .into_string()?;
        Ok(result)
    }

    fn send_message(&self, recipient: &str) -> Result<(), ureq::Error> {
        let _messege_resp = self.agent.post(&format!("{}/messages/write", self.config.server))
        .set("content-Type", "application/x-www-form-urlencoded")
        .set("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.88 Safari/537.36")
        .set("origin", "https://ts20.x2.international.travian.com")
        .set("authorization", "Bearer undefined")
        .set("cookie", &format!("{}={}", self.cookie.as_ref().unwrap().name(), self.cookie.as_ref().unwrap().value()))
        .set("referer", "https://ts20.x2.international.travian.com/messages/write")
        .send_form(&[
            ("an", recipient),
            ("be", &self.config.subject),
            ("message", &self.config.message),
    ])?;

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct Nonce {
    nonce: String,
}

#[derive(Deserialize, Debug)]
struct AppConf {
    server: String,
    login: String,
    pass: String,
    delay: u64,
    subject: String,
    message: String,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
