use std::time::Duration;

use kurosabi::kurosabi::Context;
use tokio::{io::{duplex, AsyncWriteExt}, time::sleep};

use crate::context::SiteContext;

pub struct AsciiArcAnimation;

impl AsciiArcAnimation {

    pub async fn root(mut c: Context<SiteContext>) -> Context<SiteContext> {
        c.res.text(
            "Welcome to the ASCII Art Animation!\n\
            Try accessing with curl.\n\
            Available endpoints:\n\
            /aaa/kaomoji - Watch a cute kaomoji animation.\n\
            /aaa/cat - Enjoy a playful cat animation.\n"
        );
        c
    }

    pub async fn kaomoji(mut c: Context<SiteContext>) -> Context<SiteContext> {
        if !c.req.header.get_user_agent().map_or(false, |ua| ua.contains("curl")) {
            c.res.text(
                "Access this endpoint using 'curl' to view the ASCII animation!"
            );
            return c;
        }
        const FRAMES: [&str; 2] = ["\x1b[2K\r(。-`ω-)", "\x1b[2K\r(。l`ωl)"];
        let (mut a, b) = duplex(24);

        tokio::spawn(async move {
            
            let mut idx = 0;
            loop {
                sleep(Duration::from_millis(1000)).await;
                if a.write_all(FRAMES[idx].as_bytes()).await.is_err() {
                    break;
                }
                if a.flush().await.is_err() {
                    break;
                }
                idx = (idx + 1) % FRAMES.len();
            }
        });

        // 読み取り側をそのまま AsyncRead として渡す
        let buffer_size = 24;
        c.res.header.set("Content-Type", "text/plain; charset=utf-8");
        c.res.header.set("X-Accel-Buffering", "no");
        c.res.header.set("Cache-Control", "no-cache, no-transform");
        c.res.chunked_stream(Box::pin(b), buffer_size);
        c
    }

    pub async fn cat(mut c: Context<SiteContext>) -> Context<SiteContext> {
        if !c.req.header.get_user_agent().map_or(false, |ua| ua.contains("curl")) {
            c.res.text(
                "Access this endpoint using 'curl' to view the ASCII animation!"
            );
            return c;
        }
        const FRAMES: [&str; 4] = [
            "\x1b[3A\x1b[2K\r /\\_/\\ \n\x1b[2K\r( o.o )\n\x1b[2K\r > ^ <",
            "\x1b[3A\x1b[2K\r /\\_/\\ \n\x1b[2K\r( -.- )\n\x1b[2K\r > ^ <",
            "\x1b[3A\x1b[2K\r /\\_/\\ \n\x1b[2K\r( o.o )\n\x1b[2K\r >  ^ <",
            "\x1b[3A\x1b[2K\r /\\_/\\ \n\x1b[2K\r( -.- )\n\x1b[2K\r < ^ <"
        ];
        let (mut a, b) = duplex(48);
        tokio::spawn(async move {
            
            let mut idx = 0;
            loop {
                sleep(Duration::from_millis(1000)).await;
                if a.write_all(FRAMES[idx].as_bytes()).await.is_err() {
                    break;
                }
                if a.flush().await.is_err() {
                    break;
                }
                idx = (idx + 1) % FRAMES.len();
            }
        });

        // 読み取り側をそのまま AsyncRead として渡す
        let buffer_size = 48;
        c.res.header.set("Content-Type", "text/plain; charset=utf-8");
        c.res.header.set("X-Accel-Buffering", "no");
        c.res.header.set("Cache-Control", "no-cache, no-transform");
        c.res.chunked_stream(Box::pin(b), buffer_size);
        c
    }

    pub async fn bad_apple(mut c: Context<SiteContext>) -> Context<SiteContext> {
        if !c.req.header.get_user_agent().map_or(false, |ua| ua.contains("curl")) {
            c.res.text(
                "Access this endpoint using 'curl' to view the ASCII animation!"
            );
            return c;
        }
        const FRAMES_BIN: &[u8] = include_bytes!("../../../data/aaa/bad_apple_frames.bin");
        const FRAME_SIZE: usize = 65 * 26; // width 65, height 26
        const FRAME_NUM: usize = 4385;
        const VIDEO_TIME_SEC: f32 = 219.0;
            let (mut a, b) = duplex(FRAME_SIZE + 32);
            let frame_delay = (VIDEO_TIME_SEC * 1000.0 / FRAME_NUM as f32) as u64;
            let bin_str = std::str::from_utf8(FRAMES_BIN).unwrap_or("");
            let frames: Vec<&str> = bin_str.split("\\fe\\").collect();
            tokio::spawn(async move {
                // 最初にコンソール全体クリア
                let _ = a.write_all(b"\x1b[2J\x1b[H").await;
                for (i, frame) in frames.iter().enumerate() {
                    if i >= FRAME_NUM { break; }
                    let mut ascii = String::with_capacity(frame.len() + 32);
                    ascii.push_str("\x1b[2J\x1b[H"); // クリア
                    ascii.push_str(frame);
                    ascii.push('\n');
                    if a.write_all(ascii.as_bytes()).await.is_err() { break; }
                    if a.flush().await.is_err() { break; }
                    sleep(Duration::from_millis(frame_delay)).await;
                }
            });
        c.res.header.set("Content-Type", "text/plain; charset=utf-8");
        c.res.header.set("X-Accel-Buffering", "no");
        c.res.header.set("Cache-Control", "no-cache, no-transform");
        c.res.chunked_stream(Box::pin(b), FRAME_SIZE + 128);
        c
    }
} 