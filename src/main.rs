use imap::Session;

use std::io;

mod imap_integration;

use imap_integration::ImapServer;

fn main() {
    let mut server = ImapServer::spawn();

    println!("enter folder name: ");
    let mut folder = String::new();
    io::stdin().read_line(&mut folder).expect("An error occured");

    server.change_session_selection(folder.trim());

    let number = server.fetch_uids().len();

    println!("Total in Staging: {number}");

    println!("Do you want to download? Y/N");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("input failed");

    input = input.trim().to_string();

    if input == "Y".to_string() {
        server.download_staging();
    }

    // let uid = server.fetch_uids().iter().next().unwrap().to_string();

    // server.fetch_one(uid);

    // println!("{}", server.get_totals().to_string());

    server.logout();
}
