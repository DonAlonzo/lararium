use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use serde::{Deserialize, Serialize};

pub struct MediaSource {
    pipeline: gst::Pipeline,
    video_sink: gst_app::AppSink,
    audio_sink: gst_app::AppSink,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaSample {
    pub caps: String,
    pub data: Vec<u8>,
}

impl MediaSource {
    pub fn new(file_path: &str) -> Self {
        let pipeline = gst::Pipeline::new();
        let file_src = gst::ElementFactory::make("filesrc")
            .property("location", file_path)
            .build()
            .unwrap();
        let qtdemux = gst::ElementFactory::make("qtdemux").build().unwrap();
        let video_sink = gst::ElementFactory::make("appsink")
            .name("video_sink")
            .build()
            .unwrap();
        let audio_sink = gst::ElementFactory::make("appsink")
            .name("audio_sink")
            .build()
            .unwrap();
        pipeline
            .add_many(&[&file_src, &qtdemux, &video_sink, &audio_sink])
            .unwrap();
        file_src.link(&qtdemux).unwrap();
        qtdemux.connect_pad_added({
            let pipeline = pipeline.clone();
            let video_sink = video_sink.clone();
            let audio_sink = audio_sink.clone();
            move |_, src_pad| {
                let caps = src_pad.current_caps().unwrap();
                let structure = caps.structure(0).unwrap();
                let media_type = structure.name().as_str();
                if media_type.starts_with("video/") {
                    let parser = match media_type {
                        "video/x-h264" => gst::ElementFactory::make("h264parse").build().unwrap(),
                        "video/x-h265" => gst::ElementFactory::make("h265parse").build().unwrap(),
                        "video/x-vp8" => gst::ElementFactory::make("vp8parse").build().unwrap(),
                        "video/x-vp9" => gst::ElementFactory::make("vp9parse").build().unwrap(),
                        _ => {
                            eprintln!("Unsupported codec: {}", media_type);
                            return;
                        }
                    };

                    pipeline.add(&parser).unwrap();
                    parser.sync_state_with_parent().unwrap();

                    let sink_pad = parser.static_pad("sink").unwrap();
                    src_pad.link(&sink_pad).unwrap();
                    parser.link(&video_sink).unwrap();
                } else if media_type.starts_with("audio/") {
                    let sink_pad = audio_sink.static_pad("sink").unwrap();
                    if !sink_pad.is_linked() {
                        src_pad.link(&sink_pad).unwrap();
                    }
                }
            }
        });
        let video_sink = video_sink.dynamic_cast::<gst_app::AppSink>().unwrap();
        let audio_sink = audio_sink.dynamic_cast::<gst_app::AppSink>().unwrap();
        Self {
            pipeline,
            video_sink,
            audio_sink,
        }
    }

    pub fn play(&self) {
        self.pipeline.set_state(gst::State::Playing).unwrap();
        self.video_sink.set_state(gst::State::Playing).unwrap();
        self.audio_sink.set_state(gst::State::Playing).unwrap();
    }

    pub fn pause(&self) {
        self.pipeline.set_state(gst::State::Paused).unwrap();
        self.video_sink.set_state(gst::State::Paused).unwrap();
        self.audio_sink.set_state(gst::State::Paused).unwrap();
    }

    pub fn stop(&self) {
        self.pipeline.set_state(gst::State::Null).unwrap();
    }

    pub async fn pull_video_sample(&self) -> MediaSample {
        let sample = self.video_sink.pull_sample().unwrap();
        let data = sample
            .buffer()
            .unwrap()
            .to_owned()
            .map_readable()
            .unwrap()
            .to_vec();
        let caps = sample.caps().unwrap().to_string();
        MediaSample { caps, data }
    }

    pub async fn pull_audio_sample(&self) -> MediaSample {
        let sample = self.audio_sink.pull_sample().unwrap();
        let data = sample
            .buffer()
            .unwrap()
            .to_owned()
            .map_readable()
            .unwrap()
            .to_vec();
        let caps = sample.caps().unwrap().to_string();
        MediaSample { caps, data }
    }
}

impl Drop for MediaSource {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}
