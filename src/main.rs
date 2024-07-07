use ffmpeg_next as ffmpeg;
use sdl2::pixels::PixelFormatEnum;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize FFmpeg
    ffmpeg::init()?;

    // Get the video file from command line argument
    let input_path = env::args().nth(1).expect("Please provide a path to a video file.");

    // Open the input file
    let mut ictx = ffmpeg::format::input(&input_path)?;

    // Find the best video stream
    let input = ictx.streams().best(ffmpeg::media::Type::Video).ok_or(ffmpeg::Error::StreamNotFound)?;
    let video_stream_index = input.index();
    let mut context_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(input.parameters())?;
    let mut decoder = context_decoder.decoder().video()?;

    // Set up SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem.window("Rust Video Player", decoder.width(), decoder.height())
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::IYUV,
        decoder.width(),
        decoder.height(),
    )?;

    // Read frames and display them
    let mut frame = ffmpeg::frame::Video::empty();
    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;
            while decoder.receive_frame(&mut frame).is_ok() {
                texture.update_yuv(
                    None,
                    frame.data(0), frame.stride(0) as usize,
                    frame.data(1), frame.stride(1) as usize,
                    frame.data(2), frame.stride(2) as usize,
                )?;
                canvas.clear();
                canvas.copy(&texture, None, None)?;
                canvas.present();
            }
        }
    }

    decoder.send_eof()?;
    while decoder.receive_frame(&mut frame).is_ok() {
        texture.update_yuv(
            None,
            frame.data(0), frame.stride(0) as usize,
            frame.data(1), frame.stride(1) as usize,
            frame.data(2), frame.stride(2) as usize,
        )?;
        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();
    }

    Ok(())
}
