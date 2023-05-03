use core::panic;
use std::thread;
use std::time::Duration;

use colored::Colorize;
use rand::seq::SliceRandom;
use tw_api::quran_csv::{CsvReader, Verse};
use tw_api::tw_api::{TwClient, Tweetable, TweetError};
use tw_api::tw_api::{Credentials, QuranTweet};


fn main() {
    println!("Quran Twitter bot v5...");
    println!("Getting twitter client...");
    let cred = Credentials::from_env().expect("Error loading credentials");
    let client = TwClient::new(cred);

    let verses = CsvReader::<Verse>::get("quran-dataset.csv".into()).expect("Error reading csv");
    loop {
        println!("Getting a random verse..");
        let v = verses.choose(&mut rand::thread_rng()).expect("Error getting the verse");
        // while v.ayah_en.chars().count() > 140 {
        //     println!("Verse too long, choosing another");
        //     v =verses.choose(&mut rand::thread_rng()).expect("Error getting the verse"); 
        // }
        let tweet_content = QuranTweet::from(&v);
        println!("New tweet created:");
        println!("{}", &tweet_content.fmt_tweet().text.color("blue"));
        
        if let Err(er) = client.send(tweet_content) {

            if let Some(s) = er.downcast_ref::<TweetError>() {
                println!("Tweet error: {}", s);
                thread::sleep(Duration::from_secs(100));
                continue;
            } else {
                panic!("Non tweet error!");
            }
        }

        println!("Tweet sent successfully!");
        thread::sleep(Duration::from_secs_f32(7200f32));
    }
}
