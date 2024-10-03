// NOTE: We probably want to regenerate the entire website, tags included, everytime we run this command, no?
// maybe we can then have subcommands like --update-tags or w/ever but that should be done by default probably?
// maybe we can make it so that github's CD looks only at the generated output for this?

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::string::FromUtf8Error;
use std::{fmt, fs, io};

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

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Layout::Post => write!(f, "post"),
        }
    }
}

#[derive(Debug)]
struct Tags(Vec<String>);

impl From<Vec<String>> for Tags {
    fn from(value: Vec<String>) -> Self {
        Tags(value)
    }
}

impl fmt::Display for Tags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let link_formatter =
            |tag| format!("<a class=\"link\", href=\"{tag}\"><code>{tag}</code></a>");

        let fmt = self
            .0
            .iter()
            .cloned()
            .map(link_formatter)
            .reduce(|acc, tag| format!("{acc} | {tag}"))
            .unwrap_or_default();

        let fmt = format!("<p> [ {fmt} ] </p>");

        write!(f, "{fmt}")
    }
}

#[derive(Deserialize, Debug)]
struct MetaData {
    layout: Layout,
    title: String,
    fungi: PathBuf,
    tags: Vec<String>,
}

#[derive(Debug)]
enum ParseError {
    Beginning,
    Ending,
}

#[derive(Error, Debug)]
enum Error {
    #[error("Parse error in your markdown file - do you have a proper header? (--- <header> ---)")]
    Parser(ParseError),
    #[error("Invalid yaml in your markdown header")]
    Yaml(#[from] serde_yml::Error),
    #[error("Error executing pandoc")]
    Pandoc(#[from] io::Error),
    #[error("Invalid UTF8 in pandoc output")]
    Utf8(#[from] FromUtf8Error),
}

struct Pandoc;

impl Pandoc {
    pub fn render(path: &Path) -> Result<String, Error> {
        let mut cmd = Command::new("pandoc");

        cmd.arg(path).arg("--to").arg("html").stdout(Stdio::piped());

        let child = cmd.output()?;

        Ok(String::from_utf8(child.stdout)?)
    }
}

fn parse_metadata(input: &str) -> Result<MetaData, Error> {
    let md_header = "---\n";

    if !input.starts_with(md_header) {
        return Err(Error::Parser(ParseError::Beginning));
    };

    let input = &input[4..];

    let end_pos = input
        .find(md_header)
        .ok_or(Error::Parser(ParseError::Ending))?;

    let metadata = &input[..end_pos];

    Ok(serde_yml::from_str(metadata)?)
}

fn main() -> Result<()> {
    let Args { markdown } = Args::parse();
    let html = Pandoc::render(&markdown)?;

    let content = fs::read_to_string(markdown)?;
    let MetaData {
        layout,
        title,
        fungi,
        tags,
    } = parse_metadata(&content)?;

    let tags = Tags::from(tags);

    let mut template = PathBuf::from("assets").join(layout.to_string());
    template.set_extension("tmpltl" /* FIXME: Constify */);

    let template = fs::read_to_string(dbg!(template))?;

    let template = template.replace("{{ TITLE }}", &title);
    let template = template.replace("{{ TAGS }}", &tags.to_string());
    let template = template.replace("{{ PANDOC }}", &html);
    let template = template.replace("{{ FUNGI }}", fungi.to_str().unwrap());

    println!("{template}");

    // we have certain rules because this isn't a very public framework, and we can do whatever we want
    // the first rule is that the markdown content must start with three hyphens, to indicate the post's header and metadata
    // we then parse everything until the next three hyphens
    // otherwise we error out

    Ok(())
}
