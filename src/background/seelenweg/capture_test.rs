use std::{
    io::{self, Write},
    time::Instant,
};

use windows_capture::{
    capture::GraphicsCaptureApiHandler,
    encoder::{VideoEncoder, VideoEncoderQuality, VideoEncoderType},
    frame::{Frame, ImageFormat},
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{ColorFormat, CursorCaptureSettings, DrawBorderSettings, Settings},
};

// This struct will be used to handle the capture events.
struct Capture {
    // The video encoder that will be used to encode the frames.
    encoder: Option<VideoEncoder>,
    // To measure the time the capture has been running
    start: Instant,
}

impl GraphicsCaptureApiHandler for Capture {
    // The type of flags used to get the values from the settings.
    type Flags = String;

    // The type of error that can occur during capture, the error will be returned from `CaptureControl` and `start` functions.
    type Error = Box<dyn std::error::Error + Send + Sync>;

    // Function that will be called to create the struct. The flags can be passed from settings.
    fn new(message: Self::Flags) -> Result<Self, Self::Error> {
        println!("Got The Flag: {message}");

        let encoder = VideoEncoder::new(
            VideoEncoderType::Mp4,
            VideoEncoderQuality::HD1080p,
            1920,
            1080,
            "target/video.mp4",
        )?;

        Ok(Self {
            encoder: Some(encoder),
            start: Instant::now(),
        })
    }

    // Called every time a new frame is available.
    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        print!(
            "\rRecording for: {} seconds",
            self.start.elapsed().as_secs()
        );

        println!("\r\n1");

        frame.save_as_image("target/frame.png", ImageFormat::Png).expect("Failed to save image");
        println!("\r\n1.123");

        // Send the frame to the video encoder
/*         self.encoder
            .as_mut()
            .unwrap()
            .send_frame(frame)
            .expect("Failed to send frame"); */

        println!("\r\n2");

        // Note: The frame has other uses too for example you can save a single for to a file like this:
        // frame.save_as_image("frame.png", ImageFormat::Png)?;
        // Or get the raw data like this so you have full control:
        // let data = frame.buffer()?;

        // Stop the capture after 6 seconds
        if self.start.elapsed().as_secs() >= 6 {
            // Finish the encoder and save the video.
            /* self.encoder.take().unwrap().finish()?; */

            capture_control.stop();

            // Because there wasn't any new lines in previous prints
            println!();
        }
        println!("3");

        Ok(())
    }

    // Optional handler called when the capture item (usually a window) closes.
    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("Capture Session Closed");

        Ok(())
    }
}

pub fn testing_capture() {
    std::thread::spawn(move || {
        // Gets The Foreground Window, Checkout The Docs For Other Capture Items
        let primary_monitor = Monitor::primary().expect("There is no primary monitor");

        let settings = Settings::new(
            // Item To Captue
            primary_monitor,
            // Capture Cursor Settings
            CursorCaptureSettings::Default,
            // Draw Borders Settings
            DrawBorderSettings::Default,
            // The desired color format for the captured frame.
            ColorFormat::Rgba8,
            // Additional flags for the capture settings that will be passed to user defined `new` function.
            "Yea This Works".to_string(),
        )
        .unwrap();
        // Starts the capture and takes control of the current thread.
        // The errors from handler trait will end up here
        let control = Capture::start_free_threaded(settings).expect("Screen Capture Failed");
    });
}
