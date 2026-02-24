use bevy::prelude::*;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tracing_subscriber::Layer;

const MAX_LOG_ENTRIES: usize = 50;
const FLUSH_INTERVAL_SECS: f32 = 30.0;

/// Shared ring buffer for log entries.
#[derive(Clone)]
pub struct LogRingBuffer(pub Arc<Mutex<VecDeque<String>>>);

impl Default for LogRingBuffer {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LOG_ENTRIES))))
    }
}

/// Resource wrapper for Bevy ECS access.
#[derive(Resource, Clone)]
pub struct LogBuffer(pub LogRingBuffer);

/// Timer for periodic flush.
#[derive(Resource)]
struct LogFlushTimer(Timer);

/// Custom tracing Layer that writes to the ring buffer.
pub struct RingBufferLayer {
    buffer: LogRingBuffer,
}

impl RingBufferLayer {
    pub fn new(buffer: LogRingBuffer) -> Self {
        Self { buffer }
    }
}

impl<S> Layer<S> for RingBufferLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let level = metadata.level();

        // Only capture our crate's events at DEBUG and above
        let target = metadata.target();
        if !target.starts_with("simple_platformer") {
            return;
        }

        let mut visitor = MessageVisitor(String::new());
        event.record(&mut visitor);

        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let entry = format!("[{}] {:<5} {}: {}", now, level, target, visitor.0);

        if let Ok(mut buf) = self.buffer.0.lock() {
            if buf.len() >= MAX_LOG_ENTRIES {
                buf.pop_front();
            }
            buf.push_back(entry);
        }
    }
}

struct MessageVisitor(String);

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{:?}", value);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0 = value.to_string();
        }
    }
}

fn log_path() -> std::path::PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let dir = base.join("simple_platformer");
    std::fs::create_dir_all(&dir).ok();
    dir.join("game.log")
}

fn flush_to_file(buffer: &LogRingBuffer) {
    if let Ok(buf) = buffer.0.lock() {
        let content: String = buf.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n");
        let path = log_path();
        std::fs::write(path, content).ok();
    }
}

fn periodic_flush(
    time: Res<Time>,
    mut timer: ResMut<LogFlushTimer>,
    log_buffer: Res<LogBuffer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        flush_to_file(&log_buffer.0);
    }
}

fn flush_on_exit(
    mut exit_events: bevy::ecs::message::MessageReader<AppExit>,
    log_buffer: Res<LogBuffer>,
) {
    for _ in exit_events.read() {
        flush_to_file(&log_buffer.0);
    }
}

pub struct LoggingPlugin;

impl Plugin for LoggingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LogFlushTimer(Timer::from_seconds(
            FLUSH_INTERVAL_SECS,
            TimerMode::Repeating,
        )))
        .add_systems(Update, (periodic_flush, flush_on_exit));
    }
}

/// Call from main() BEFORE App::new() to set up tracing subscriber.
/// Returns the ring buffer for insertion into the Bevy app.
pub fn setup_tracing() -> LogRingBuffer {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::EnvFilter;

    let ring_buffer = LogRingBuffer::default();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer().with_filter(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "simple_platformer=info,wgpu=error,bevy=warn".parse().unwrap()),
            ),
        )
        .with(RingBufferLayer::new(ring_buffer.clone()))
        .init();

    ring_buffer
}
