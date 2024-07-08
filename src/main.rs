use ffmpeg_next as ffmpeg;
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum, render::{Canvas, Texture}, EventPump};
use std::{env, error::Error, time::{Duration, Instant}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

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

    // Calculate the frame delay
    let time_base = input.time_base();
    let frame_rate: f64 = input.avg_frame_rate().into();
    log::debug!("frame rate: {:#?}", frame_rate);
    let frame_delay = Duration::from_secs_f64(1. / frame_rate);

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
    let mut start_time = Instant::now();

    // Set up event handling
    let mut event_pump = sdl_context.event_pump()?;
    let mut running = true;

    // Read frames and display them
    let mut frame = ffmpeg::frame::Video::empty();
    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;
            let _ = action(
                &mut frame, 
                &mut decoder, 
                &mut event_pump, 
                &mut running, 
                &mut texture, 
                &mut canvas,
                &mut start_time,
                &frame_delay,
            );
        }
    }

    decoder.send_eof()?;
    let _ = action(
        &mut frame, 
        &mut decoder, 
        &mut event_pump, 
        &mut running, 
        &mut texture, 
        &mut canvas, 
        &mut start_time,
        &frame_delay,
     );

    Ok(())
}

fn action(
    frame: &mut ffmpeg::frame::Video,
    decoder: &mut  ffmpeg::decoder::Video,
    event_pump: &mut  EventPump,
    running: &mut bool,
    texture: &mut Texture,
    canvas: &mut  Canvas<sdl2::video::Window>,
    start_time: &mut Instant,
    frame_delay: &Duration,
) -> Result<(), Box<dyn Error>> {
    while decoder.receive_frame(frame).is_ok() {
        for event in event_pump.poll_iter() {
            //log::info!("event: {:#?}", event);
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    *running = false;
                    break;
                },
                _ => {}
            }
        }

        if !*running {
            break;
        }

        texture.update_yuv(
            None,
            frame.data(0), frame.stride(0) as usize,
            frame.data(1), frame.stride(1) as usize,
            frame.data(2), frame.stride(2) as usize,
        )?;
        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();

        // Calculate elapsed time and sleep if necessary to match framerate
        let elapsed = start_time.elapsed();
        if elapsed < *frame_delay {
            let sleep_time = *frame_delay - elapsed;
            log::debug!("sleep time: {:#?}", sleep_time);
            std::thread::sleep(sleep_time);
        }
        *start_time = Instant::now();
    }

    Ok(())
}
