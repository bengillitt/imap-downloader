use imap;
use native_tls;

use std::time::Duration;

use mailparse::parse_mail;

use mailparse::MailHeaderMap;

use std::net::TcpStream;

use dotenv::dotenv;

use std::env;

pub struct ImapServer {
    session: imap::Session<native_tls::TlsStream<TcpStream>>,
}

impl ImapServer {
    pub fn spawn() -> ImapServer {
        dotenv().ok();

        let domain = "imap.mail.yahoo.com";
        let tls = native_tls::TlsConnector::builder().build().unwrap();

        let client = imap::connect((domain, 993), domain, &tls).unwrap();

        let mut imap_session = client
            .login(
                env::var("EMAIL").unwrap(),
                env::var("EMAIL_PASSWORD").unwrap(),
            )
            .map_err(|e| e.0)
            .unwrap();

        println!(
            "Inbox exists: {}",
            imap_session.examine("INBOX").unwrap().exists
        );
        println!(
            "Inbox recent: {}",
            imap_session.examine("INBOX").unwrap().recent
        );

        // imap_session.select("INBOX");

        // let messages = imap_session.fetch("1", "RFC822").unwrap();

        // let message = if let Some(m) = messages.iter().next() {
        //     m
        // } else {
        //     panic!("No message found");
        // };

        // let body = message.body().unwrap();

        // let body = std::str::from_utf8(body).unwrap().to_string();

        // println!("{}", body);

        // println!("{}", imap_session.examine("INBOX").unwrap().exists);

        // let mailboxes = imap_session.list(None, Some("*")).unwrap();

        // for mailbox in &mailboxes {
        //     maconnecttch imap_session.select(mailbox.name()) {
        //         Ok(mail) => (),
        //         Err(_) => continue,
        //     }

        //     let uids = imap_session.uid_search("ALL").unwrap();
        //     println!("Name: {0}, Number: {1}", mailbox.name(), uids.len());
        // }

        return ImapServer {
            session: imap_session,
        };
    }

    pub fn fetch_one(&mut self, uid: String) {
        self.session.select("INBOX");

        let messages = self.session.uid_fetch(uid, "RFC822").unwrap();

        let message = if let Some(m) = messages.iter().next() {
            m
        } else {
            panic!("No message found");
        };

        let body = message.body().unwrap();

        let body = std::str::from_utf8(body).unwrap().to_string();

        println!("{}", body);
    }

    pub fn fetch_uids(&mut self) -> std::collections::HashSet<imap::types::Uid> {
        return match self.session.uid_search("ALL") {
            Ok(hashset) => hashset,
            Err(e) => {
                eprintln!("uid_search err: {e}");
                std::collections::HashSet::new()
            }
        };
    }

    pub fn get_totals(&mut self) -> u32 {
        let folders = self.session.list(None, Some("*")).unwrap();

        let mut total: u32 = 0;

        for folder in &folders {
            match self.session.select(folder.name()) {
                Ok(_) => (),
                Err(_) => continue,
            };

            let number = self.fetch_uids().len();

            println!("Name: {0}, Number: {1}", folder.name(), number);

            total += number as u32;
        }

        return total;
    }

    pub fn change_session_selection(&mut self, folder: &str) {
        self.session.select(folder).expect("selecting Failed");
    }

    pub fn logout(&mut self) {
        self.session.logout();
    }

    pub fn download_staging(&mut self) {
        std::fs::create_dir_all("./emails");

        let total = self.fetch_uids().len() as u32;

        let mut start = 1u32;
        let batch_size = 100u32;

        while start <= total {
            let end = (start + batch_size - 1).min(total);

            let messages = self
                .session
                .fetch(format!("{}:{}", start, end), "RFC822")
                .unwrap();

            for message in messages.iter() {
                let uid = message.uid.unwrap_or(start);
                let body = match message.body() {
                    Some(b) => b,
                    None => continue,
                };

                let path = format!("./emails/{}", ImapServer::make_filename(uid, body));
                std::fs::write(&path, body);
                println!("Saved {}", path);
                std::thread::sleep(Duration::from_millis(100));
            }

            start = end + 1;
        }
    }

    fn make_filename(uid: u32, raw: &[u8]) -> String {
        let parsed = parse_mail(raw).unwrap();
        let headers = parsed.get_headers();

        let date = headers
            .get_first_value("Date")
            .and_then(|d| mailparse::dateparse(&d).ok())
            .map(|ts| {
                chrono::DateTime::from_timestamp(ts, 0)
                    .unwrap()
                    .format("%Y-%m-%d")
                    .to_string()
            })
            .unwrap_or("0000-00-00".to_string());

        let subject = headers
            .get_first_value("Subject")
            .unwrap_or("No Subject".to_string());

        let subject = ImapServer::sanitize_subject(&subject, 50);

        format!("{}_{}_{:08}.eml", date, subject, uid)
    }

    fn sanitize_subject(subject: &str, max_len: usize) -> String {
        subject
            .chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
                c if c.is_control() => '-',
                c => c,
            })
            .filter(|c| c.is_ascii()) // strip non-ascii for safety
            .take(max_len)
            .collect::<String>()
            .trim()
            .to_string()
    }
}
