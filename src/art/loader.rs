//! Application-owned native SVG worker. No global worker pool or async runtime.
use super::{MAX_CACHE_BYTES, MAX_CACHE_SOURCE_BYTES, MAX_RASTER_SIZE, Svg, SvgError, parse, rasterize_tree};
use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use std::{
    collections::VecDeque,
    sync::{Arc, mpsc},
    task::Poll,
    thread,
};

const MAX_PENDING: usize = 2;
const MAX_ENTRIES: usize = 32;
type Key = (Arc<str>, [u32; 2]);
type Ticket = Arc<()>;

#[derive(Debug)]
struct Job {
    key: Key,
    ticket: Ticket,
}

#[derive(Debug)]
struct Completed {
    job: Job,
    image: Result<ColorImage, SvgError>,
}

enum State {
    Pending(Ticket),
    Ready(Result<TextureHandle, SvgError>),
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending(_) => f.write_str("Pending"),
            Self::Ready(result) => f.debug_tuple("Ready").field(&result.as_ref().map(TextureHandle::id)).finish(),
        }
    }
}

#[derive(Debug)]
struct Entry {
    key: Key,
    state: State,
    bytes: usize,
}

struct WakeOnExit(Context);

impl Drop for WakeOnExit {
    fn drop(&mut self) {
        self.0.request_repaint();
    }
}

/// A bounded SVG loader for native applications. Keep one instance in application
/// state and reuse it across frames. One worker parses and rasterizes; [`Self::load`]
/// polls without waiting and uploads completed images through the owning context.
///
/// At most two jobs are accepted at once (one running, one queued), with at most
/// 4 MiB of source text in the retained entries. The ready cache retains at most
/// 32 entries and 32 MiB of RGBA textures. A raster is at most 2048 × 2048 pixels.
/// Sources larger than 4 MiB are rejected before copying. Failed documents are
/// cached too. Equal source content and dimensions share a result, regardless of name.
///
/// Source copying, cache lookup and texture upload still occur on the calling
/// thread. These limits do not bound SVG tree/filter memory or parsing complexity.
/// Dropping or clearing the loader discards results without waiting; it cannot
/// interrupt an active resvg call. Cleared in-flight jobs may temporarily retain
/// their sources and images until completion, outside the ready-cache limits.
///
/// ```
/// # #[cfg(not(target_arch = "wasm32"))] {
/// use spaceui::{art::SvgLoader, Icon, egui};
/// use std::task::Poll;
/// let ctx = egui::Context::default();
/// let mut loader = SvgLoader::new(&ctx)?;
/// match loader.load(Icon::Orbit.svg(), [64, 32])? {
///     Poll::Ready(texture) => { /* Paint with texture.id(). */ }
///     Poll::Pending => { /* Reserve space or paint a placeholder. */ }
/// }
/// # }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug)]
pub struct SvgLoader {
    ctx: Context,
    jobs: mpsc::SyncSender<Job>,
    completed: mpsc::Receiver<Completed>,
    entries: VecDeque<Entry>,
    stopped: bool,
    in_flight: usize,
    in_flight_source_bytes: usize,
    // Dropping a JoinHandle detaches; never join an expensive SVG from the UI.
    _worker: thread::JoinHandle<()>,
}

impl SvgLoader {
    /// Start a single worker bound to `ctx`. Completion wakes the egui integration.
    ///
    /// # Errors
    /// Returns the operating system error if the worker thread cannot be started.
    pub fn new(ctx: &Context) -> std::io::Result<Self> {
        Self::spawn(ctx, |source, size| rasterize_tree(&parse(source)?, size))
    }

    fn spawn(
        ctx: &Context,
        mut render: impl FnMut(&str, [u32; 2]) -> Result<ColorImage, SvgError> + Send + 'static,
    ) -> std::io::Result<Self> {
        let (jobs, requests) = mpsc::sync_channel::<Job>(1);
        let (results, completed) = mpsc::sync_channel(MAX_PENDING);
        let wake = ctx.clone();
        let worker = thread::Builder::new().name("spaceui-svg".into()).spawn(move || {
            // A worker panic must wake the UI too, so it can observe WorkerStopped.
            let _wake_on_exit = WakeOnExit(wake.clone());
            while let Ok(job) = requests.recv() {
                let image = render(&job.key.0, job.key.1);
                if results.send(Completed { job, image }).is_err() {
                    break;
                }
                wake.request_repaint();
            }
        })?;
        Ok(Self {
            ctx: ctx.clone(),
            jobs,
            completed,
            entries: VecDeque::new(),
            stopped: false,
            in_flight: 0,
            in_flight_source_bytes: 0,
            _worker: worker,
        })
    }

    /// Request or poll an exact pixel size. Ready handles stay valid if the cache
    /// is cleared or evicted. There is no synchronous parsing/rasterization fallback.
    /// The caller chooses the placeholder while pending and any retry/error display.
    ///
    /// # Errors
    /// Returns [`SvgError::InvalidSize`] or [`SvgError::SourceTooLarge`] before
    /// enqueueing invalid requests; [`SvgError::QueueFull`] is retryable. Worker
    /// failures and document/raster errors are distinct from [`Poll::Pending`].
    pub fn load(&mut self, svg: Svg<'_>, size: [u32; 2]) -> Result<Poll<TextureHandle>, SvgError> {
        for dimension in size {
            if !(1..=MAX_RASTER_SIZE).contains(&dimension) {
                return Err(SvgError::InvalidSize(dimension));
            }
        }
        if svg.source.len() > MAX_CACHE_SOURCE_BYTES {
            return Err(SvgError::SourceTooLarge(svg.source.len()));
        }
        self.poll_completed();
        if let Some(index) =
            self.entries.iter().position(|entry| entry.key.0.as_ref() == svg.source && entry.key.1 == size)
            && let Some(entry) = self.entries.remove(index)
        {
            let result = match &entry.state {
                State::Pending(_) => Ok(Poll::Pending),
                State::Ready(result) => result.clone().map(Poll::Ready),
            };
            self.entries.push_back(entry);
            return result;
        }
        if self.stopped {
            return Err(SvgError::WorkerStopped);
        }
        if self.in_flight >= MAX_PENDING || self.in_flight_source_bytes + svg.source.len() > MAX_CACHE_SOURCE_BYTES {
            return Err(SvgError::QueueFull);
        }
        let key = (Arc::from(svg.source), size);
        let ticket = Arc::new(());
        self.jobs.try_send(Job { key: key.clone(), ticket: ticket.clone() }).map_err(|error| match error {
            mpsc::TrySendError::Full(_) => SvgError::QueueFull,
            mpsc::TrySendError::Disconnected(_) => {
                self.stopped = true;
                SvgError::WorkerStopped
            }
        })?;
        self.in_flight += 1;
        self.in_flight_source_bytes += svg.source.len();
        self.entries.push_back(Entry { key, state: State::Pending(ticket), bytes: 0 });
        self.trim();
        Ok(Poll::Pending)
    }

    /// Discard retained handles/errors and pending tickets. Old jobs cannot fill
    /// the new cache, even if the same source/size is immediately requested again.
    /// Active work is not interrupted, and may keep the queue busy until it finishes.
    pub fn clear(&mut self) {
        self.entries.clear();
        // Discard completed images without uploading them.
        while let Ok(completed) = self.completed.try_recv() {
            self.in_flight -= 1;
            self.in_flight_source_bytes -= completed.job.key.0.len();
        }
    }

    fn poll_completed(&mut self) {
        loop {
            match self.completed.try_recv() {
                Ok(completed) => {
                    self.in_flight -= 1;
                    self.in_flight_source_bytes -= completed.job.key.0.len();
                    let Some(entry) = self.entries.iter_mut().find(|entry| {
                        matches!(&entry.state, State::Pending(ticket) if Arc::ptr_eq(ticket, &completed.job.ticket))
                    }) else { continue; };
                    entry.state = State::Ready(completed.image.map(|image| {
                        entry.bytes = image.size[0] * image.size[1] * 4;
                        self.ctx.load_texture("spaceui-background-svg", image, TextureOptions::LINEAR)
                    }));
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.stopped = true;
                    self.in_flight = 0;
                    self.in_flight_source_bytes = 0;
                    for entry in &mut self.entries {
                        if matches!(entry.state, State::Pending(_)) {
                            entry.state = State::Ready(Err(SvgError::WorkerStopped));
                        }
                    }
                    break;
                }
            }
        }
        self.trim();
    }

    fn trim(&mut self) {
        while self.entries.len() > MAX_ENTRIES
            || self.entries.iter().map(|entry| entry.key.0.len()).sum::<usize>() > MAX_CACHE_SOURCE_BYTES
            || self.entries.iter().map(|entry| entry.bytes).sum::<usize>() > MAX_CACHE_BYTES
        {
            let Some(index) = self.entries.iter().position(|entry| matches!(entry.state, State::Ready(_))) else {
                break;
            };
            self.entries.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    const SOURCE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10"><path d="M0 0H20V10H0Z" fill="white"/></svg>"#;

    fn finish(loader: &mut SvgLoader, svg: Svg<'_>, size: [u32; 2]) -> Result<TextureHandle, SvgError> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match loader.load(svg, size) {
                Ok(Poll::Ready(texture)) => return Ok(texture),
                Ok(Poll::Pending) | Err(SvgError::QueueFull) => {}
                Err(error) => return Err(error),
            }
            assert!(Instant::now() < deadline, "worker did not complete");
            thread::yield_now();
        }
    }

    #[test]
    fn work_runs_off_thread_queue_is_bounded_and_clear_discards_stale_results() {
        let ctx = Context::default();
        let caller = thread::current().id();
        let (entered, started) = mpsc::channel();
        let (release, gate) = mpsc::channel();
        let mut loader = SvgLoader::spawn(&ctx, move |_, size| {
            assert_ne!(thread::current().id(), caller);
            entered.send(()).unwrap();
            gate.recv_timeout(Duration::from_secs(10)).unwrap();
            Ok(ColorImage::filled(size.map(|n| n as usize), egui::Color32::WHITE))
        })
        .unwrap();
        let svg = Svg::new("wide", SOURCE);
        assert!(loader.load(svg, [20, 10]).unwrap().is_pending());
        started.recv_timeout(Duration::from_secs(10)).unwrap();
        loader.clear();
        assert!(loader.load(svg, [20, 10]).unwrap().is_pending());
        assert!(matches!(loader.load(svg, [30, 10]), Err(SvgError::QueueFull)));
        assert_eq!(loader.in_flight, MAX_PENDING, "clear must not reset outstanding-work accounting");
        release.send(()).unwrap();
        started.recv_timeout(Duration::from_secs(10)).unwrap();
        // The old result has arrived, but the replacement job is still blocked.
        assert!(loader.load(svg, [20, 10]).unwrap().is_pending());
        assert_eq!(loader.in_flight, 1);
        release.send(()).unwrap();
        let texture = finish(&mut loader, svg, [20, 10]).unwrap();
        assert_eq!(texture.size(), [20, 10]);
        assert_eq!(finish(&mut loader, Svg::new("another name", SOURCE), [20, 10]).unwrap().id(), texture.id());
        loader.clear();
        assert_eq!(texture.size(), [20, 10], "external handle survives clear");
    }

    #[test]
    fn asynchronous_pixels_match_synchronous_raster_and_runtime_edits_invalidate() {
        let ctx = Context::default();
        let mut loader = SvgLoader::new(&ctx).unwrap();
        let original = Svg::new("art", SOURCE);
        let expected = original.rasterize_size([40, 40]).unwrap();
        let texture = finish(&mut loader, original, [40, 40]).unwrap();
        let output = ctx.run_ui(egui::RawInput::default(), |_| {});
        let delta = &output.textures_delta.set[&texture.id()][0];
        let egui::ImageData::Color(image) = &delta.image;
        assert_eq!(image.as_ref(), &expected);
        output.drop_without_applying_deltas();
        let edited = SOURCE.replace("white", "black");
        assert_ne!(finish(&mut loader, Svg::new("art", &edited), [40, 40]).unwrap().id(), texture.id());
        assert_ne!(finish(&mut loader, original, [80, 40]).unwrap().id(), texture.id());
        assert!(matches!(finish(&mut loader, Svg::new("bad", "<svg"), [16, 16]), Err(SvgError::InvalidDocument(_))));
        assert!(matches!(loader.load(Svg::new("bad", "<svg"), [16, 16]), Err(SvgError::InvalidDocument(_))));
    }

    #[test]
    fn invalid_sizes_and_sources_are_rejected_before_work_is_enqueued() {
        let mut loader = SvgLoader::new(&Context::default()).unwrap();
        for size in [[0, 10], [10, MAX_RASTER_SIZE + 1]] {
            assert!(matches!(loader.load(Svg::new("bad", "<svg"), size), Err(SvgError::InvalidSize(_))));
        }
        assert!(matches!(
            loader.load(Svg::new("large", &" ".repeat(MAX_CACHE_SOURCE_BYTES + 1)), [1, 1]),
            Err(SvgError::SourceTooLarge(_))
        ));
        assert_eq!(loader.in_flight, 0);
        assert!(loader.entries.is_empty());
    }

    #[test]
    fn ready_cache_evicts_entries_source_text_and_raster_bytes() {
        let ctx = Context::default();
        let mut loader = SvgLoader::new(&ctx).unwrap();
        for i in 0..40 {
            let source = format!("{SOURCE}<!--{i}-->");
            let _ = finish(&mut loader, Svg::new("art", &source), [4, 4]).unwrap();
        }
        assert_eq!(loader.entries.len(), MAX_ENTRIES);
        let source = format!("{SOURCE}<!--{}-->", " ".repeat(MAX_CACHE_SOURCE_BYTES / 2));
        let _ = finish(&mut loader, Svg::new("large", &source), [4, 4]).unwrap();
        let _ = finish(&mut loader, Svg::new("large", &format!("{source} ")), [4, 4]).unwrap();
        assert!(loader.entries.iter().map(|entry| entry.key.0.len()).sum::<usize>() <= MAX_CACHE_SOURCE_BYTES);
        assert_eq!(loader.entries.len(), 1);
        loader.clear();
        for i in 0..3 {
            let _ = finish(&mut loader, Svg::new("art", &format!("{SOURCE}<!--{i}-->")), [2048, 2048]).unwrap();
            ctx.run_ui(egui::RawInput::default(), |_| {}).drop_without_applying_deltas();
        }
        assert_eq!(loader.entries.len(), 2);
        assert_eq!(loader.entries.iter().map(|entry| entry.bytes).sum::<usize>(), MAX_CACHE_BYTES);
    }

    #[test]
    fn a_stopped_worker_is_an_error_instead_of_permanent_pending() {
        let ctx = Context::default();
        let mut loader = SvgLoader::spawn(&ctx, |_, _| panic!("simulated worker failure")).unwrap();
        assert!(matches!(finish(&mut loader, Svg::new("art", SOURCE), [4, 4]), Err(SvgError::WorkerStopped)));
        assert!(matches!(loader.load(Svg::new("other", "<svg"), [4, 4]), Err(SvgError::WorkerStopped)));
    }

    #[test]
    fn dropped_loader_does_not_join_a_blocked_worker() {
        let (entered, started) = mpsc::channel();
        let (release, gate) = mpsc::channel();
        let mut loader = SvgLoader::spawn(&Context::default(), move |_, _| {
            entered.send(()).unwrap();
            gate.recv_timeout(Duration::from_secs(10)).unwrap();
            Err(SvgError::AllocationFailed)
        })
        .unwrap();
        assert!(loader.load(Svg::new("art", SOURCE), [4, 4]).unwrap().is_pending());
        started.recv_timeout(Duration::from_secs(10)).unwrap();
        drop(loader);
        release.send(()).unwrap();
    }
}
