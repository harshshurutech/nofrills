use askama::Template;
use clap::Parser;
use pulldown_cmark::{Options, Parser as MarkdownParser, html};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

/// A CLI tool to convert Markdown files into HTML slideshows
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short = 'i', long, help = "Input markdown file")]
    input: PathBuf,

    #[clap(
        short = 'o',
        long,
        help = "Output HTML file (omit for stdout with --stdout)"
    )]
    output: Option<PathBuf>,

    #[clap(long, help = "Output HTML to stdout instead of a file")]
    stdout: bool,
}

#[derive(Template)]
#[template(path = "template.html", escape = "none")]
struct SlideshowTemplate {
    slides_content: String,
    total_slides: usize,
}

/// Convert markdown to HTML using pulldown-cmark
fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = MarkdownParser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let md_content = fs::read_to_string(&args.input)?;

    let slides: Vec<&str> = md_content.split("---").map(|s| s.trim()).collect();

    let slides_html = slides
        .iter()
        .enumerate()
        .map(|(i, slide)| {
            let html_content = markdown_to_html(slide);
            format!(
                "<section id=\"slide-{}\" class=\"slide\">{}</section>",
                i + 1,
                html_content
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    // Create the HTML template
    let template = SlideshowTemplate {
        slides_content: slides_html,
        total_slides: slides.len(),
    };

    let html = template.render()?;

    if args.stdout {
        println!("{}", html);
    } else if let Some(output_path) = args.output {
        let mut file = File::create(&output_path)?;
        file.write_all(html.as_bytes())?;
        println!("Successfully generated slideshow at: {:?}", output_path);
    } else {
        return Err("Error: Must specify --output or --stdout".into());
    }

    Ok(())
}
