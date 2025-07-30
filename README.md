# 🎥 MyVideoCompressor

A simple command-line tool written in Rust to **compress video files to fit under Discord's 10MB upload limit**.

It automatically adjusts compression quality using `ffmpeg` to hit the size target while preserving as much quality as possible.

---

## ✅ Features

- 🔧 Automatic CRF tuning (quality/size trade-off)
- 📦 Ensures output is < 10MB (for Discord)
- 📥 Auto-downloads `ffmpeg` (Windows only)
- 🔁 Retries with lower quality if the video is too large
- 🛠 Easy to run from command line (context menu support coming soon)

---

## 🧪 Example usage

```sh
cargo run -- "path/to/video.mp4"
