// NOTE: We probably want to regenerate the entire website, tags included, everytime we run this command, no?
// maybe we can then have subcommands like --update-tags or w/ever but that should be done by default probably?
// maybe we can make it so that github's CD looks only at the generated output for this?

use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use serde::Deserialize;
use thiserror::Error;

#[derive(Parser, Clone)]
struct Args {
    #[arg(short, long, help = "Post content")]
    markdown: PathBuf,
}

#[derive(Deserialize, Debug)]
enum Layout {
    #[serde(rename = "post")]
    Post,
}

#[derive(Deserialize, Debug)]
struct MetaData {
    layout: Layout,
    title: String,
    tags: Vec<String>,
}

#[derive(Debug)]
enum ParseError {
    Beginning,
}

#[derive(Error, Debug)]
enum Error {
    #[error("Parse error in your markdown file - do you have a proper header? (--- <header> ---)")]
    Parser(ParseError),
    #[error("Invalid yaml in your markdown header")]
    Yaml(#[from] serde_yml::Error),
}

fn parse_metadata(input: &str) -> Result<MetaData, Error> {
    let md_header = "---\n";

    if !input.starts_with(md_header) {
        return Err(Error::Parser(ParseError::Beginning));
    };

    let end_pos = dbg!(input[4..].find(md_header).unwrap());

    let metadata = &input[4..end_pos];

    Ok(serde_yml::from_str(dbg!(metadata))?)
}

fn main() -> Result<()> {
    let Args { markdown } = Args::parse();

    let content = fs::read_to_string(markdown)?;
    let meta = parse_metadata(&content)?;

    dbg!(meta);

    // we have certain rules because this isn't a very public framework, and we can do whatever we want
    // the first rule is that the markdown content must start with three hyphens, to indicate the post's header and metadata
    // we then parse everything until the next three hyphens
    // otherwise we error out

    Ok(())
}
