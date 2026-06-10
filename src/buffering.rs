use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use kit_config::BufferingConfig;

use crate::event::LogEvent;
use crate::output::Output;
use crate::formatter::Formatter;

pub struct Buffer {
    sender: Sender<LogEvent>,
    handle: Option<thread::JoinHandle<()>>,
    running: Arc<Mutex<bool>>,
    outputs: Arc<Mutex<Vec<Box<dyn Output>>>>,
    #[allow(dead_code)]
    formatter: Arc<Mutex<Box<dyn Formatter>>>,
}

impl Buffer {
    pub fn new(
        config: BufferingConfig,
        outputs: Vec<Box<dyn Output>>,
        formatter: Box<dyn Formatter>,
    ) -> Self {
        let (sender, receiver) = mpsc::channel::<LogEvent>();
        let running = Arc::new(Mutex::new(true));
        let running_clone = running.clone();
        let outputs_arc = Arc::new(Mutex::new(outputs));
        let formatter_arc = Arc::new(Mutex::new(formatter));
        let outputs_clone = outputs_arc.clone();
        let formatter_clone = formatter_arc.clone();

        let batch_size = config.batch_size;
        let flush_interval = Duration::from_millis(config.flush_interval_ms);

        let handle = Some(thread::spawn(move || {
            let mut batch: Vec<LogEvent> = Vec::with_capacity(batch_size);

            loop {
                let running = *running_clone.lock().unwrap();
                if !running && batch.is_empty() {
                    break;
                }

                if let Ok(event) = receiver.recv_timeout(flush_interval) {
                    batch.push(event);

                    if batch.len() >= batch_size {
                        flush_batch(&batch, &outputs_clone, &formatter_clone);
                        batch.clear();
                    }
                } else if !batch.is_empty() {
                    flush_batch(&batch, &outputs_clone, &formatter_clone);
                    batch.clear();
                }

                if !running && batch.is_empty() {
                    break;
                }
            }

            if !batch.is_empty() {
                flush_batch(&batch, &outputs_clone, &formatter_clone);
            }
        }));

        Self {
            sender,
            handle,
            running,
            outputs: outputs_arc,
            formatter: formatter_arc,
        }
    }

    pub fn send(&self, event: LogEvent) {
        let _ = self.sender.send(event);
    }

    pub fn flush(&self) {
        let outputs = self.outputs.lock().unwrap();
        for output in outputs.iter() {
            let _ = output.flush();
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        *self.running.lock().unwrap() = false;
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn flush_batch(
    batch: &[LogEvent],
    outputs: &Arc<Mutex<Vec<Box<dyn Output>>>>,
    formatter: &Arc<Mutex<Box<dyn Formatter>>>,
) {
    let outputs = outputs.lock().unwrap();
    let formatter = formatter.lock().unwrap();
    for event in batch {
        for output in outputs.iter() {
            let _ = output.write(event, &**formatter);
        }
    }
    for output in outputs.iter() {
        let _ = output.flush();
    }
}
