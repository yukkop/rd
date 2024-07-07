extern crate ffmpeg_next as ffmpeg;
extern crate rodio;

use ffmpeg::{format, media};
use ffmpeg::media::Type;
use std::slice::SliceIndex;
use rodio::Source;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Open video file
    let mut ictx = *format::input(&"your_video.mp4")?;
    let mut decoder = ictx.streams().best(media::Type::Video).unwrap().codec().decoder().unwrap();

    // Audio playback example (using rodio)
    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
    let sink = stream_handle.play_raw();

    // Example audio (replace with your own audio loading logic)
    let source = rodio::source::SineWave::new(440.0);
    sink.append(source);

    // Display video frames (example)
    let mut packet = ffmpeg::Packet::empty();
    while ictx.read_packet(&mut packet)? {
        if packet.stream().index() == decoder.index() {
            let mut frame = ffmpeg::Frame::empty();
            decoder.decode(&packet, &mut frame)?;

            // Display or process video frame here (e.g., with OpenGL or image crate)

            // Example: Print frame information
            println!("Frame number: {}", decoder.frames_decoded());
        }
    }

    Ok(())
}
