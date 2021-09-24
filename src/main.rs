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

// The story and its rank on the HN page
#[derive(Debug)]
struct Entry {
    rank: usize,
    story: Story,
}

#[derive(Debug, Deserialize)]
struct Story {
    by: String,
    descendants: u32,
    id: u32,
    score: u32,
    title: String,
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!(
        "Hacker news downloader.\nUsage: {} [-n <number>] [-o]",
        program
    );
    print!("{}", opts.usage(&brief));
}

fn read_story_ids<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<u32>> {
    let text = fs::read_to_string(path)?;
    let story_ids = text
        .split('\n')
        .map(|s| s.trim().parse::<u32>().unwrap())
        .collect();
    Ok(story_ids)
}

fn write_story_ids<P: AsRef<Path>>(path: P, story_ids: &[u32]) -> anyhow::Result<()> {
    let text = story_ids
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join("\n");
    fs::write(path, text)?;
    Ok(())
}

// Download a single story from the HN API.
fn fetch_story(id: u32) -> anyhow::Result<Story> {
    let story =
        reqwest::blocking::get(format!("{api}/item/{id}.json", api = API, id = id))?.json()?;
    Ok(story)
}

fn fetch_top_stories(num: usize, known_story_ids: &[u32]) -> Vec<Entry> {
    // Get the Ids of the top N stories fronm HN.
    let stories_top_ids: Vec<u32> =
        reqwest::blocking::get(format!("{api}/topstories.json", api = API))
            .unwrap()
            .json::<Vec<u32>>()
            .unwrap()
            .into_iter()
            .take(num)
            .collect();

    // Filter out the stories we already know.
    // Keep the rank.
    let mut entries: Vec<Entry> = Vec::new();
    for (rank, id) in stories_top_ids.iter().enumerate() {
        if known_story_ids.contains(id) {
            continue;
        }
        if let Ok(story) = fetch_story(*id) {
            entries.push(Entry { rank, story });
        }
    }
    entries
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
    let story_ids_fn = config_dir.join("story_ids.lst");

    if !only_new {
        match fs::remove_file(&story_ids_fn) {
            Err(_) => println!("Could not remove entries list"),
            Ok(_) => println!("Removed entries list"),
        }
    }

    println!("Fetching {} stories...", number);
    let mut story_ids: Vec<u32> = Vec::new();
    if only_new {
        println!("(downloading only new stories)");
        if let Ok(sids) = read_story_ids(&story_ids_fn) {
            story_ids = sids;
        } else {
            println!("Could not read entries list.");
        }
    }

    let entries = fetch_top_stories(number, &story_ids);

    // Add the new entries to the list of stories we already know:
    story_ids.extend(entries.iter().map(|e| e.story.id).collect::<Vec<u32>>());
    match write_story_ids(&story_ids_fn, &story_ids) {
        Err(e) => println!("Could not write story ids: {}", e),
        Ok(_) => println!("Wrote story ids"),
    }

    for entry in entries.iter() {
        println!(
            "{}",
            format!("{:2}. {}", entry.rank + 1, entry.story.title).bold()
        );
        println!(
            "    score: {score}    comments: {comments}    user: {user}",
            score = entry.story.score,
            comments = entry.story.descendants,
            user = entry.story.by
        );
        let url = format!(
            "    url: https://news.ycombinator.com/item?id={id}",
            id = entry.story.id
        );
        println!("{}\n", url.dimmed());
    }
}
