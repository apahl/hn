// Hacker News title downloader
// Inspired by the `V` version: https://github.com/BafS/hn-top

use colored::Colorize;
use getopts::Options;
use serde::Deserialize;
use std::env;

const API: &str = "https://hacker-news.firebaseio.com/v0";

#[derive(Debug, Deserialize)]
struct Story {
    by: String,
    descendants: u32,
    kids: Vec<u32>,
    id: u32,
    score: u32,
    // time        int
    title: String,
    #[serde(rename = "type")]
    typ: String,
    url: String,
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Hacker news downloader.\nUsage: {} [-n <number>]", program);
    print!("{}", opts.usage(&brief));
}

fn fetch_story(id: u32) -> anyhow::Result<Story> {
    let story =
        reqwest::blocking::get(format!("{api}/item/{id}.json", api = API, id = id))?.json()?;
    Ok(story)
}

fn fetch_top_stories(num: usize) -> Vec<Story> {
    let stories_top_ids: Vec<u32> =
        reqwest::blocking::get(format!("{api}/topstories.json", api = API))
            .unwrap()
            .json::<Vec<u32>>()
            .unwrap()
            .into_iter()
            .take(num)
            .collect();

    let stories = stories_top_ids
        .iter()
        .filter_map(|id| fetch_story(*id).ok())
        .collect::<Vec<Story>>();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt(
        "n",
        "",
        "the number of entries to download (1-99, default: 10)",
        "NUMBER",
    );
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            println!("\n{}\n", "Error parsing commandline arguments.".red());
            print_usage(&program, opts);
            return;
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let number = matches
        .opt_str("n")
        .unwrap_or_else(|| String::from("10"))
        .parse::<usize>()
        .unwrap_or(10)
        .max(1)
        .min(99);

    println!("Fetching {} stories...", number);
    let stories = fetch_top_stories(number);
    for (idx, story) in stories.iter().enumerate() {
        println!(
            "{}. {}",
            format!("{:2}", idx + 1).bold(),
            story.title.bold()
        );
        println!(
            "    score: {score}    comments: {comments}    user: {user}",
            score = story.score,
            comments = story.descendants,
            user = story.by
        );
        let url = format!(
            "    url: https://news.ycombinator.com/item?id={id}",
            id = story.id
        );
        println!("{}\n", url.dimmed());
    }
}
