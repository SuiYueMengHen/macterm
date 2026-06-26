use std::sync::Arc;
use std::sync::RwLock;

use anyhow::{Context, Result};
use macterm_core::PaneId;
use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use tokio::sync::mpsc;

/// Events emitted by a PTY session
#[derive(Debug, Clone)]
pub enum PtyEvent {
    Output(PaneId, String),
    Resized(PaneId, u16, u16),
    Exited(PaneId, i32),
}

/// Represents a PTY session with vt100 terminal emulation
pub struct PtySession {
    pub pane_id: PaneId,
    /// The master PTY handle — kept alive for `resize()` (SIGWINCH to shell)
    master: Box<dyn MasterPty + Send>,
    pub writer: Box<dyn std::io::Write + Send>,
    pub reader: Option<tokio::task::JoinHandle<()>>,
    pub parser: Arc<RwLock<vt100::Parser>>,
    #[allow(dead_code)]
    parser_tx: mpsc::UnboundedSender<PtyEvent>,
}

impl PtySession {
    /// Spawn a new shell in a PTY
    pub fn spawn(
        pane_id: PaneId,
        cols: u16,
        rows: u16,
        #[allow(dead_code)]
        parser_tx: mpsc::UnboundedSender<PtyEvent>,
    ) -> Result<Self> {
        let pty_system = NativePtySystem::default();

        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to open PTY")?;
        let (master, slave) = (pair.master, pair.slave);

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        let mut cmd = CommandBuilder::new(shell);
        cmd.env("TERM", "xterm-256color");
        let mut child = slave
            .spawn_command(cmd)
            .context("Failed to spawn shell in PTY")?;

        // Get writer from the master — does NOT consume it in portable-pty 0.9
        let writer = master.take_writer()?;
        let reader = master.try_clone_reader()?;

        let parser = Arc::new(RwLock::new(vt100::Parser::new(rows, cols, 10000)));

        let parser_clone = parser.clone();
        let pid = pane_id;
        let tx = parser_tx.clone();

        let reader_handle = tokio::task::spawn_blocking(move || {
            let mut buf = [0u8; 4096];
            let mut reader = reader;
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        let _ = tx.send(PtyEvent::Exited(pid, 0));
                        break;
                    }
                    Ok(n) => {
                        if let Ok(mut p) = parser_clone.write() {
                            p.process(&buf[..n]);
                        }
                        let _ = tx.send(PtyEvent::Output(pid, String::new()));
                    }
                    Err(e) => {
                        log::error!("PTY read error for pane {}: {}", pid, e);
                        let _ = tx.send(PtyEvent::Exited(pid, 1));
                        break;
                    }
                }
            }
        });

        // Spawn child reaper
        let _child_handle = tokio::spawn(async move {
            match child.wait() {
                Ok(status) => {
                    log::info!("Shell exited with code {:?}", status.exit_code());
                }
                Err(e) => {
                    log::error!("Failed to wait for shell: {}", e);
                }
            }
        });

        Ok(Self {
            pane_id,
            master,
            writer,
            reader: Some(reader_handle),
            parser,
            parser_tx,
        })
    }

    /// Write data to the PTY (user input)
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.writer
            .write_all(data)
            .context("Failed to write to PTY")?;
        self.writer.flush()?;
        Ok(())
    }

    /// Resize the PTY and its vt100 parser.
    /// Updates both the in-memory parser AND the kernel PTY size,
    /// so the shell process receives SIGWINCH and redraws at the new size.
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        if let Ok(mut p) = self.parser.write() {
            p.set_size(rows, cols);
        }
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }
}
