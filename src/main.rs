// Hacker News title downloader
// Inspired by the `V` version: https://github.com/BafS/hn-top

use colored::Colorize;
use directories::ProjectDirs;
use getopts::Options;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

const API: &str = "https://hacker-news.firebaseio.com/v0";

#[derive(Debug, Deserialize)]
struct Story {
    by: String,
    descendants: u32,
    // kids: Vec<u32>,
    id: u32,
    score: u32,
    // time        int
    title: String,
    // #[serde(rename = "type")]
    // typ: String,
    // url: String,
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Hacker news downloader.\nUsage: {} [-n <number>]", program);
    print!("{}", opts.usage(&brief));
}

fn read_entries_list<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<u32>> {
    let text = fs::read_to_string(path)?;
    let entries = text
        .split('\n')
        .map(|s| s.trim().parse::<u32>().unwrap())
        .collect();
    Ok(entries)
}

fn write_entries_list<P: AsRef<Path>>(path: P, entries: &[u32]) -> anyhow::Result<()> {
    let text = entries
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join("\n");
    fs::write(path, text)?;
    Ok(())
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
    stories
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
    opts.optflag("o", "onlynew", "only download new entries");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            println!("\n{}\n", "Error parsing commandline arguments.".red());
            print_usage(&program, opts);
            return;
        }
    };
    let only_new = matches.opt_present("o");
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

    let proj_dir = ProjectDirs::from("com", "github.apahl", "hn").unwrap();
    let config_dir = proj_dir.config_dir();
    fs::create_dir_all(config_dir).unwrap();
    let entry_fn = config_dir.join("entries.lst");
    if !only_new {
        match fs::remove_file(&entry_fn) {
            Err(_) => println!("Could not remove entries list"),
            Ok(_) => println!("Removed entries list"),
        }
    }
    println!("Fetching {} stories...", number);
    let mut stories = fetch_top_stories(number);
    let mut entries_list: Vec<u32> = vec![];
    if only_new {
        println!("(downloading only new stories)");
        if let Ok(el) = read_entries_list(&entry_fn) {
            // stories = stories.iter().filter(|s| !entries_list.contains(&s.id));
            entries_list = el;
            stories.retain(|s| !entries_list.contains(&s.id));
        } else {
            println!("Could not read entries list.");
        }
    }
    entries_list.extend(stories.iter().map(|s| s.id).collect::<Vec<u32>>());
    match write_entries_list(&entry_fn, &entries_list) {
        Err(e) => println!("Could not write entries list: {}", e),
        Ok(_) => println!("Wrote entries list"),
    }
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
