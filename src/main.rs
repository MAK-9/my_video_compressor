use clap::Parser;
use std::path::PathBuf;

/// Compress a video to under 10MB for Discord
#[derive(Parser, Debug)]
#[command(name = "vid-compress")]
#[command(about = "Compress a video for Discord (<10MB)", long_about = None)]
struct Args
{
    /// Input video file
    input: PathBuf,
}

fn main()
{
    let args = Args::parse();

    println!("Input file path: {:?}", args.input);
}