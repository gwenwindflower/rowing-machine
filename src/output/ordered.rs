use std::collections::BTreeMap;
use std::sync::mpsc::{SyncSender, sync_channel};
use std::thread::{Builder, JoinHandle};

use anyhow::{Context, Result};

use super::{OutputSink, PreparedUnit};

enum Message {
    Unit(PreparedUnit, usize),
    Barrier(SyncSender<()>),
}

pub(crate) struct OrderedOutput {
    sender: SyncSender<Message>,
    writer: JoinHandle<Result<BTreeMap<String, u64>>>,
}

impl OrderedOutput {
    pub(crate) fn new(mut sink: OutputSink, capacity: usize) -> Result<Self> {
        let (sender, receiver) = sync_channel(capacity);
        let writer = Builder::new()
            .name("output".into())
            .spawn(move || {
                for message in receiver {
                    match message {
                        Message::Unit(unit, remaining) => sink.write_unit(unit, remaining)?,
                        Message::Barrier(ready) => {
                            let _ = ready.send(());
                        }
                    }
                }
                sink.finish()
            })
            .context("starting ordered output writer")?;
        Ok(Self { sender, writer })
    }

    pub(crate) fn write(&self, unit: PreparedUnit, remaining: usize) -> Result<()> {
        self.sender
            .send(Message::Unit(unit, remaining))
            .map_err(|_| anyhow::anyhow!("output writer stopped"))
    }

    pub(crate) fn complete_stage(&self) -> Result<()> {
        let (sender, receiver) = sync_channel(0);
        self.sender
            .send(Message::Barrier(sender))
            .map_err(|_| anyhow::anyhow!("output writer stopped"))?;
        receiver
            .recv()
            .context("output writer stopped before completing stage")
    }

    pub(crate) fn finish(self) -> Result<BTreeMap<String, u64>> {
        drop(self.sender);
        self.writer
            .join()
            .map_err(|_| anyhow::anyhow!("output writer panicked"))?
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use super::*;
    use crate::output::{
        Column, ColumnType, EntitySchema, EntityWriter, Format, Row, UnitEncoder, Value,
    };

    struct SlowWriter {
        started: SyncSender<()>,
        release: std::sync::mpsc::Receiver<()>,
        bytes: Arc<Mutex<Vec<u8>>>,
    }

    impl EntityWriter for SlowWriter {
        fn write_row(&mut self, _: &Row) -> Result<()> {
            unreachable!()
        }
        fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
            if self.bytes.lock().unwrap().is_empty() {
                self.started.send(())?;
                self.release.recv_timeout(Duration::from_secs(10))?;
            }
            self.bytes.lock().unwrap().extend_from_slice(bytes);
            Ok(())
        }
        fn finish(self: Box<Self>) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn later_units_can_be_handed_off_while_an_earlier_write_is_blocked() {
        let directory = tempfile::tempdir().unwrap();
        let schema = EntitySchema {
            name: "records",
            columns: vec![Column {
                name: "id",
                column_type: ColumnType::Integer,
                nullable: false,
            }],
            primary_key: vec!["id"],
        };
        let encoder = UnitEncoder::new(vec![schema.clone()], Format::Csv);
        let mut sink = OutputSink::new(directory.path(), "raw", vec![schema]).unwrap();
        let (started, waiting) = sync_channel(0);
        let (release, blocked) = sync_channel(0);
        let bytes = Arc::new(Mutex::new(Vec::new()));
        sink.entities.get_mut("records").unwrap().writer = Some(Box::new(SlowWriter {
            started,
            release: blocked,
            bytes: Arc::clone(&bytes),
        }));
        let output = OrderedOutput::new(sink, 1).unwrap();
        let unit = |id| {
            encoder
                .encode(&vec![("records", vec![Value::Integer(id)])])
                .unwrap()
        };
        output.write(unit(1), 2).unwrap();
        waiting.recv_timeout(Duration::from_secs(10)).unwrap();
        output.write(unit(2), 1).unwrap();
        release.send(()).unwrap();
        output.complete_stage().unwrap();
        assert_eq!(output.finish().unwrap()["records"], 2);
        assert_eq!(*bytes.lock().unwrap(), b"1\n2\n");
    }
}
