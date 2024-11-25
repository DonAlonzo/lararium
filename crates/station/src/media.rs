use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub struct MediaSink {
    video_pipeline: gst::Pipeline,
    video_src: gst_app::AppSrc,
    audio_pipeline: gst::Pipeline,
    audio_src: gst_app::AppSrc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaSample {
    pub caps: String,
    pub data: Vec<u8>,
}

impl MediaSink {
    pub fn new(use_wayland: bool) -> Self {
        let video_pipeline = gst::Pipeline::new();
        let video_src = gst::ElementFactory::make("appsrc")
            .name("video_src")
            .build()
            .unwrap();
        let video_decode = gst::ElementFactory::make("decodebin")
            .name("video_decode")
            .build()
            .unwrap();
        let video_queue = gst::ElementFactory::make("queue")
            .name("video_queue")
            .property("max-size-time", 3_000_000_000u64)
            .build()
            .unwrap();
        let video_convert = gst::ElementFactory::make("videoconvert").build().unwrap();
        let video_scale = gst::ElementFactory::make("videoscale").build().unwrap();
        let video_sink = if use_wayland {
            gst::ElementFactory::make("waylandsink").build().unwrap()
        } else {
            gst::ElementFactory::make("kmssink")
                //.property("bus-id", "PCI:0000:01:00.0")
                .build()
                .unwrap()
        };
        video_pipeline
            .add_many([
                &video_src,
                &video_decode,
                &video_queue,
                &video_convert,
                &video_scale,
                &video_sink,
            ])
            .unwrap();
        let video_src = video_src.dynamic_cast::<gst_app::AppSrc>().unwrap();
        video_src.set_stream_type(gst_app::AppStreamType::Stream);
        video_src.set_format(gst::Format::Time);
        //video_src.set_property("is-live", true);
        //video_src.set_property("do-timestamp", true);
        video_src.link(&video_decode).unwrap();
        video_decode.connect_pad_added({
            let video_queue = video_queue.clone();
            move |_, src_pad| {
                let sink_pad = video_queue.static_pad("sink").unwrap();
                if sink_pad.is_linked() {
                    return;
                }
                src_pad.link(&sink_pad).unwrap();
            }
        });
        video_queue.sync_state_with_parent().unwrap();
        video_queue.link(&video_convert).unwrap();
        video_convert.link(&video_scale).unwrap();
        video_scale.link(&video_sink).unwrap();
        video_sink.set_property("sync", true);

        let audio_pipeline = gst::Pipeline::new();
        let audio_src = gst::ElementFactory::make("appsrc")
            .name("audio_src")
            .build()
            .unwrap();
        let audio_decode = gst::ElementFactory::make("decodebin")
            .name("audio_decode")
            .build()
            .unwrap();
        let audio_queue = gst::ElementFactory::make("queue")
            .name("audio_queue")
            .property("max-size-time", 3_000_000_000u64)
            .build()
            .unwrap();
        let audio_convert = gst::ElementFactory::make("audioconvert").build().unwrap();
        let audio_resample = gst::ElementFactory::make("audioresample").build().unwrap();
        let audio_sink = gst::ElementFactory::make("alsasink").build().unwrap();
        audio_pipeline
            .add_many([
                &audio_src,
                &audio_decode,
                &audio_queue,
                &audio_convert,
                &audio_resample,
                &audio_sink,
            ])
            .unwrap();
        let audio_src = audio_src.dynamic_cast::<gst_app::AppSrc>().unwrap();
        audio_src.set_stream_type(gst_app::AppStreamType::Stream);
        audio_src.set_format(gst::Format::Time);
        //audio_src.set_property("is-live", true);
        //audio_src.set_property("do-timestamp", true);
        audio_src.link(&audio_decode).unwrap();
        audio_decode.connect_pad_added({
            let audio_queue = audio_queue.clone();
            move |_, src_pad| {
                let sink_pad = audio_queue.static_pad("sink").unwrap();
                if sink_pad.is_linked() {
                    return;
                }
                src_pad.link(&sink_pad).unwrap();
            }
        });
        audio_queue.sync_state_with_parent().unwrap();
        audio_queue.link(&audio_convert).unwrap();
        audio_convert.link(&audio_resample).unwrap();
        audio_resample.link(&audio_sink).unwrap();
        audio_sink.set_property("sync", true);

        Self {
            video_pipeline,
            video_src,
            audio_pipeline,
            audio_src,
        }
    }

    pub fn play(&self) {
        self.video_pipeline.set_state(gst::State::Playing).unwrap();
        self.audio_pipeline.set_state(gst::State::Playing).unwrap();
    }

    pub fn pause(&self) {
        self.video_pipeline.set_state(gst::State::Paused).unwrap();
        self.audio_pipeline.set_state(gst::State::Paused).unwrap();
    }

    pub fn stop(&self) {
        self.video_pipeline.set_state(gst::State::Null).unwrap();
        self.audio_pipeline.set_state(gst::State::Null).unwrap();
    }

    pub fn push_video_sample(
        &self,
        sample: MediaSample,
    ) {
        let caps = gst::Caps::from_str(&sample.caps).unwrap();
        self.video_src.set_caps(Some(&caps));
        let buffer = gst::Buffer::from_slice(sample.data);
        let _ = self.video_src.push_buffer(buffer);
    }

    pub fn push_audio_sample(
        &self,
        sample: MediaSample,
    ) {
        let caps = gst::Caps::from_str(&sample.caps).unwrap();
        self.audio_src.set_caps(Some(&caps));
        let buffer = gst::Buffer::from_slice(sample.data);
        let _ = self.audio_src.push_buffer(buffer);
    }
}

impl Drop for MediaSink {
    fn drop(&mut self) {
        let _ = self.video_pipeline.set_state(gst::State::Null);
        let _ = self.audio_pipeline.set_state(gst::State::Null);
    }
}
