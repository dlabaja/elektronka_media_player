use std::fs::{File};
use std::*;
use eventual::Timer;
use std::io::{stdout, Write};
use std::path::Path;
use image::{AnimationDecoder, Frame};
use std::process::{Command, ExitStatus};
use crossterm::{cursor, queue, QueueableCommand, style, terminal};
use crossterm::style::{Color, Print};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use image::codecs::gif::GifDecoder;
use rodio::{Decoder, OutputStream, Source};

const VIDEO_FORMATS: [&str; 9] = ["mp4", "m4v", "mkv", "webm", "mov", "avi", "wmv", "mpg", "flw"];

fn main() {
    //parse first arg as path
    let args = env::args().collect::<Vec<String>>();
    let path = args.get(1).unwrap().trim();

    //check if the file is supported
    if File::open(Path::new(&path)).is_err() || !is_video(Path::new(&path)) {
        panic!("Invalid path or unsupported file!")
    }

    //check ffmpeg
    Command::new("ffmpeg").output().expect("FFMPEG NOT FOUND! Please install one at https://ffmpeg.org/");
    println!("Processing video (this might take some time)\n------------------");

    //convert video
    println!("Converting video");
    let video = &format!("{}{}output.gif", dirs::cache_dir().unwrap().display(), get_system_backslash());
    Command::new("ffmpeg").args(["-i", path, "-vf", "scale=80:45,fps=20", &Path::new(&video).display().to_string(), "-y"]).spawn().expect("Unable to convert to gif").wait().unwrap();

    //convert audio
    println!("Converting audio");
    let audio = &format!("{}{}output.mp3", dirs::cache_dir().unwrap().display(), get_system_backslash());
    Command::new("ffmpeg").args(["-i", &Path::new(&path).display().to_string(), &format!("{}", Path::new(audio).display()), "-y"]).spawn().expect("Unable to convert audio").wait().unwrap();

    //switch to alternate screen and modify cursor
    enable_raw_mode().unwrap();
    let mut stdout = stdout();
    queue!(
            stdout,
            terminal::EnterAlternateScreen,
            cursor::DisableBlinking,
            cursor::Hide,
            ).unwrap();
    stdout.flush().unwrap();

    //push frames to memory
    println!("Processing frames");
    let frames = GifDecoder::new(File::open(&video).unwrap()).unwrap().into_frames().collect_frames().unwrap();

    //play sound
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let source = Decoder::new(File::open(audio).unwrap()).unwrap();
    stream_handle.play_raw(source.convert_samples()).unwrap();

    //iterate frames
    let timer = Timer::new();
    let ticks = timer.interval_ms(50).iter(); //1000 / FPS = 50

    for i in ticks.enumerate() {
        generate_frame(frames.get(i.0).ok_or_else(|| end_process("End of playback")).unwrap());
    }
}

fn end_process(msg: &str) {
    disable_raw_mode().unwrap();
    let mut stdout = stdout();
    queue!(
            stdout,
            terminal::LeaveAlternateScreen,
            cursor::EnableBlinking,
            cursor::Show,
            style::SetForegroundColor(Color::White)
        ).unwrap();
    stdout.flush().unwrap();
    println!("{}", msg);
    process::exit(0);
}

fn generate_frame(frame: &Frame) {
    let mut stdout = stdout();

    for (y, line) in frame.buffer().chunks(frame.buffer().width() as usize * 4).enumerate() { //lines
        let mut pixels = "".to_string();
        for pixel in line.chunks(4) { //pixels in line
            pixels += &format!("\x1b[38;2;{};{};{}m██", pixel[0], pixel[1], pixel[2]);
        }
        stdout.queue(cursor::MoveTo(0, y as u16)).unwrap().queue(Print(&pixels)).unwrap();
    }
    stdout.flush().unwrap();//
}

fn is_video(path: &Path) -> bool {
    VIDEO_FORMATS.contains(&path.extension().expect("No extension").to_str().unwrap())
}


fn get_system_backslash() -> &'static str {
    if cfg!(windows) {
        return "\\";
    }
    "/"
}