use clap::Parser;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(env, long, default_value_t = (Ipv6Addr::UNSPECIFIED, 8081).into())]
    listen_address: SocketAddr,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    init_tracing(&[("lararium_curator", "info")]);
    gst::init()?;

    let file_path = "century.mp4";

    let pipeline = gst::Pipeline::new();
    let file_src = gst::ElementFactory::make("filesrc")
        .property("location", file_path)
        .build()?;
    let qtdemux = gst::ElementFactory::make("qtdemux").build()?;
    let video_sink = gst::ElementFactory::make("appsink")
        .name("video_sink")
        .build()?;
    let audio_sink = gst::ElementFactory::make("appsink")
        .name("audio_sink")
        .build()?;
    pipeline.add_many(&[&file_src, &qtdemux, &video_sink, &audio_sink])?;
    file_src.link(&qtdemux)?;
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
    pipeline.set_state(gst::State::Playing)?;
    video_sink.set_state(gst::State::Playing)?;
    audio_sink.set_state(gst::State::Playing)?;

    let mut video_stream = loop {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 42000);
        match TcpStream::connect(address).await {
            Ok(stream) => break stream,
            Err(error) => {
                println!("Failed to connect: {error}. Retrying...");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    };

    let mut audio_stream = loop {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 42001);
        match TcpStream::connect(address).await {
            Ok(stream) => break stream,
            Err(error) => {
                println!("Failed to connect: {error}. Retrying...");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    };

    tokio::spawn({
        let video_sink = video_sink.clone();
        async move {
            loop {
                match video_sink.pull_sample() {
                    Ok(sample) => {
                        let buffer = sample
                            .buffer()
                            .expect("Failed to get buffer from sample")
                            .to_owned();
                        let data = buffer
                            .map_readable()
                            .expect("Failed to map buffer readable");
                        let caps = sample.caps().expect("Failed to get caps from sample");
                        let length = data.len() as u32;
                        let caps_str = caps.to_string();
                        let caps_len = caps_str.len() as u32;
                        video_stream
                            .write_all(&caps_len.to_be_bytes())
                            .await
                            .unwrap();
                        video_stream.write_all(caps_str.as_bytes()).await.unwrap();
                        video_stream.write_all(&length.to_be_bytes()).await.unwrap();
                        video_stream.write_all(&data).await.unwrap();
                    }
                    Err(err) => {
                        eprintln!("Error pulling sample: {:?}", err);
                        break;
                    }
                }
            }
        }
    });

    tokio::spawn({
        let audio_sink = audio_sink.clone();
        async move {
            loop {
                match audio_sink.pull_sample() {
                    Ok(sample) => {
                        let buffer = sample
                            .buffer()
                            .expect("Failed to get buffer from sample")
                            .to_owned();
                        let data = buffer
                            .map_readable()
                            .expect("Failed to map buffer readable");
                        let caps = sample.caps().expect("Failed to get caps from sample");
                        let length = data.len() as u32;
                        let caps_str = caps.to_string();
                        let caps_len = caps_str.len() as u32;
                        audio_stream
                            .write_all(&caps_len.to_be_bytes())
                            .await
                            .unwrap();
                        audio_stream.write_all(caps_str.as_bytes()).await.unwrap();
                        audio_stream.write_all(&length.to_be_bytes()).await.unwrap();
                        audio_stream.write_all(&data).await.unwrap();
                    }
                    Err(err) => {
                        eprintln!("Error pulling sample: {:?}", err);
                        break;
                    }
                }
            }
        }
    });

    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        match msg.view() {
            gst::MessageView::Eos(..) => break,
            gst::MessageView::Error(err) => {
                eprintln!(
                    "Error received from element {:?}: {:?}",
                    err.src().map(|s| s.path_string()),
                    err.error()
                );
                eprintln!("Debugging information: {:?}", err.debug());
                break;
            }
            _ => (),
        }
    }
    pipeline.set_state(gst::State::Null)?;

    Ok(())
}

fn init_tracing(filter: &[(&str, &str)]) {
    let filter = filter
        .iter()
        .map(|(name, level)| format!("{}={}", name, level))
        .collect::<Vec<_>>()
        .join(",");
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .init();
}
