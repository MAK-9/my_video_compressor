use clap::Parser;
use std::{fs, io::Cursor, path::PathBuf, path::Path, process::Command};
use zip::ZipArchive;
use tempfile::NamedTempFile;
use reqwest::blocking::Client;
use std::time::Duration;


const MAX_SIZE_BYTES: u64 = 10 * 1024/*Kilo*/ * 1024/*Mega*/; // 10MB
const MAX_RETRIES: u8 = 6; // Try crf 28,30,32,34,36,38

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
    ensure_ffmpeg_exists();
    
    let args = Args::parse();
    
    if !args.input.exists()
    {
        eprintln!("❌ Input file does not exist: {}", args.input.display());
        std::process::exit(1);
    }

    let output = generate_output_path(&args.input);
    
    println!("✅ Input: {}", args.input.display());
    println!("📁 Output: {}", output.display());
    compress_video(&args.input, &output);
}

fn generate_output_path(input: &PathBuf) -> PathBuf
{
    let stem = input.file_stem().unwrap().to_string_lossy();
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!("{}_discord.mp4", stem))
}


fn compress_video(input: &PathBuf, output: &PathBuf) 
{
    let ffmpeg = ensure_ffmpeg_exists();  // This function returns the path to ffmpeg.exe

    let mut crf = 28;
    for attempt in 0..=MAX_RETRIES
    {
        println!("🔧 Attempt {} with CRF={}", attempt + 1, crf);

        let status = Command::new(&ffmpeg)
        .args([
            "-i", input.to_str().unwrap(),
            "-vcodec", "libx264",
            "-crf", &crf.to_string(),
            "-preset", "fast",
            "-acodec", "aac",
            "-b:a", "128k",
            "-movflags", "+faststart",
            "-y",
            output.to_str().unwrap(),
        ])
        .stdout(std::process::Stdio::null()) // suppress normal output
        .stderr(std::process::Stdio::null()) // suppress errors unless we print manually
        .status();
    

        if let Err(e) = status 
        {
            eprintln!("❌ ffmpeg failed: {}", e);
            return;
        }

        if !status.unwrap().success() 
        {
            eprintln!("❌ ffmpeg exited with error.");
            return;
        }

        let metadata = fs::metadata(output).expect("Failed to get output metadata");
        let size = metadata.len();
        let original_size = fs::metadata(input).expect("Failed to get input metadata").len();
        println!("📦 Output file size: {:.2} MB", size as f64 / 1_048_576.0);

        if size <= MAX_SIZE_BYTES 
        {
            println!("✅ Output is under 10MB, done! - Reduced from {:.2} MB to {:.2} MB.", original_size as f64 / 1_048_576.0, size as f64 / 1_048_576.0);
            return;
        } 
        else 
        {
            if crf >= 40 {
                eprintln!("⚠️ Reached max CRF but file is still too large.");
                return;
            }
            crf += 2; // Increase CRF to lower quality and size
        }
    }
}


fn ensure_ffmpeg_exists() -> PathBuf {
    if let Ok(path) = which::which("ffmpeg") {
        return path;
    }

    // Search recursively in "bin/" for "ffmpeg.exe"
    if let Some(found) = find_ffmpeg_in_bin() {
        return found;
    }

    println!("🔍 ffmpeg not found. Downloading static Windows build...");

    let download_url = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip";

    let mut temp_zip = NamedTempFile::new().expect("Failed to create temp file");
    let bytes = download_ffmpeg(download_url).expect("Failed to download ffmpeg");

    std::io::copy(&mut Cursor::new(bytes), &mut temp_zip).expect("Failed to write to temp file");

    println!("📦 Extracting ffmpeg...");

    let mut archive = ZipArchive::new(temp_zip.reopen().unwrap()).expect("Failed to open zip");

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => PathBuf::from("bin").join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }
    }

        // After downloading and extracting, try searching again
        if let Some(found) = find_ffmpeg_in_bin() {
            return found;
        }

        panic!("❌ Failed to locate ffmpeg.exe even after download");
    }

fn find_ffmpeg_in_bin() -> Option<PathBuf> {
    let bin_dir = Path::new("bin");

    if !bin_dir.exists() {
        return None;
    }

    let walker = walkdir::WalkDir::new(bin_dir).into_iter();

    for entry in walker.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() && path.file_name().map(|n| n == "ffmpeg.exe").unwrap_or(false) {
            return Some(path.to_path_buf());
        }
    }

    None
}

fn download_ffmpeg(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(120)) // 2 min timeout
        .build()?;

    for attempt in 1..=3 {
        println!("📥 Downloading ffmpeg (attempt {attempt})...");
        match client.get(url).send() {
            Ok(resp) if resp.status().is_success() => {
                let bytes = resp.bytes()?.to_vec();
                println!("✅ Download complete");
                return Ok(bytes);
            }
            Ok(resp) => println!("⚠️ Server returned HTTP {}", resp.status()),
            Err(e) => println!("⚠️ Download error: {e}"),
        }
        println!("🔄 Retrying in 3s...");
        std::thread::sleep(Duration::from_secs(3));
    }

    Err("❌ Failed to download ffmpeg after 3 attempts".into())
}
