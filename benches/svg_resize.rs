//! Frame costs for changing raster sizes, with cold and reusable documents.
use spaceui::{Icon, Svg, art, egui};
use std::time::Instant;

fn measure(name: &str, assets: &[Svg<'_>], cold: bool, frames: u32) {
    let ctx = egui::Context::default();
    let draw = |px: u32| {
        ctx.run_ui(egui::RawInput::default(), |ui| {
            for asset in assets {
                let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(px as f32));
                asset.try_paint(ui.painter(), rect, egui::Color32::WHITE).unwrap();
            }
        })
        .drop_without_applying_deltas();
    };
    draw(16); // Warm fonts and, when supported, the parsed documents.
    let mut samples = Vec::new();
    for frame in 0..frames {
        if cold {
            art::clear_cache(&ctx);
        }
        let start = Instant::now();
        draw(32 + frame); // Every measured frame needs a new raster size.
        samples.push(start.elapsed().as_secs_f64() * 1_000_000.0);
    }
    samples.sort_by(f64::total_cmp);
    println!(
        "{name:30} median {:9.2} us  p95 {:9.2} us  max {:9.2} us  n={frames}",
        samples[samples.len() / 2],
        samples[samples.len() * 95 / 100],
        samples[samples.len() - 1]
    );
    println!("  retained: {:?}", art::cache_stats(&ctx));
}

#[cfg(not(target_arch = "wasm32"))]
fn background() {
    use std::{hint::black_box, task::Poll};
    let ctx = egui::Context::default();
    let mut loader = art::SvgLoader::new(&ctx).unwrap();
    let source = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">{}</svg>"#,
        r#"<circle cx="128" cy="128" r="120" fill="white" fill-opacity="0.01"/>"#.repeat(5000)
    );
    let svg = Svg::new("complex", &source);
    let mut sync = Vec::new();
    let mut enqueue = Vec::new();
    let mut max_poll = Vec::new();
    let mut latency = Vec::new();
    for _ in 0..30 {
        let start = Instant::now();
        black_box(svg.try_rasterize(256).unwrap());
        sync.push(start.elapsed().as_secs_f64() * 1_000_000.0);
        loader.clear();
        let start = Instant::now();
        assert!(loader.load(svg, [256, 256]).unwrap().is_pending());
        enqueue.push(start.elapsed().as_secs_f64() * 1_000_000.0);
        let mut longest = 0.0_f64;
        loop {
            let poll_start = Instant::now();
            let result = loader.load(svg, [256, 256]).unwrap();
            longest = longest.max(poll_start.elapsed().as_secs_f64() * 1_000_000.0);
            if let Poll::Ready(texture) = result {
                let _ = black_box(texture);
                break;
            }
            std::thread::yield_now();
        }
        max_poll.push(longest);
        latency.push(start.elapsed().as_secs_f64() * 1_000_000.0);
        ctx.run_ui(egui::RawInput::default(), |_| {}).drop_without_applying_deltas();
    }
    for (label, mut samples) in [
        ("complex SVG sync raster", sync),
        ("background enqueue call", enqueue),
        ("background slowest poll/job", max_poll),
        ("background end-to-end", latency),
    ] {
        samples.sort_by(f64::total_cmp);
        println!(
            "{label:30} median {:9.2} us  p95 {:9.2} us  n={}",
            samples[samples.len() / 2],
            samples[samples.len() * 95 / 100],
            samples.len()
        );
    }
}

fn main() {
    let icons: Vec<_> = Icon::ALL.iter().map(|icon| icon.svg()).collect();
    let logo = Svg::new("resize/logo", include_str!("../examples/assets/spaceui.svg"));
    for (label, assets, frames) in [
        ("one icon", &icons[..1], 200),
        ("70 icons", &icons[..], 80),
        ("gallery logo", std::slice::from_ref(&logo), 200),
    ] {
        measure(&format!("{label}, cold"), assets, true, frames);
        measure(&format!("{label}, resize"), assets, false, frames);
    }
    #[cfg(not(target_arch = "wasm32"))]
    background();
}
