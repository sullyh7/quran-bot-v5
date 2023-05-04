use core::panic;
use std::thread;
use std::time::Duration;

use colored::Colorize;
use rand::seq::SliceRandom;
use tw_api::quran_csv::{CsvReader, Verse};
use tw_api::tw_api::{TwClient, Tweetable, TweetError};
use tw_api::tw_api::{Credentials, QuranTweet};


fn main() {
    println!("Welcome to Quran Twitter bot v5");
    println!("Getting twitter client...");
    let cred = Credentials::from_env().expect("Error loading credentials");
    let client = TwClient::new(cred);
    println!("{}", String::from("Client created successfully").color("green"));
    let verses = CsvReader::<Verse>::get("quran-dataset.csv".into()).expect("Error reading csv");
    const MAX_LEN: usize = 170;
    loop {
        println!("Getting a random verse..");
        let mut v = verses.choose(&mut rand::thread_rng()).expect("Error getting the verse");
        while v.ayah_en.chars().count() > MAX_LEN || v.ayah_ar.chars().count() > MAX_LEN {
            println!("Verse too long, choosing another");
            v =verses.choose(&mut rand::thread_rng()).expect("Error getting the verse"); 
        }
        let tweet_content = QuranTweet::from(&v);
        println!("New tweet created:");
        println!("\n{}", &tweet_content.fmt_tweet().text.color("blue"));
        println!("{}\n", &tweet_content.fmt_tweet_tr(String::new()).text.color("blue"));
        
        if let Err(er) = client.send(tweet_content) {
            match er.downcast_ref::<TweetError>() {
                Some(e) => {
                    println!("Tweet error: {}", e);
                thread::sleep(Duration::from_secs(100));
                continue;
                },
                None => panic!("Non tweet posting error: {}", er)
            }
        }

        println!("{}", String::from("Tweet sent successfully!").color("green"));
        thread::sleep(Duration::from_secs_f32(7200f32));
    }
}
