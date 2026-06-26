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

        // Apply Tokyo Night color scheme (F2) — feed OSC sequences to the parser
        // Sets default fg/bg and ANSI color palette 0-15 for a cohesive theme.
        if let Ok(mut p) = parser.write() {
            // OSC 10/11: default foreground/background
            p.process(b"\x1b]10;rgb:a9a9/b1b1/d6d6\x1b\\"); // fg: #a9b1d6
            p.process(b"\x1b]11;rgb:1a1b/1b1b/2626\x1b\\"); // bg: #1a1b26
            // OSC 4: ANSI color palette (Tokyo Night)
            // Black / Dark colors
            p.process(b"\x1b]4;0;rgb:3232/3232/3f3f\x1b\\");   // black:   #32323f
            p.process(b"\x1b]4;1;rgb:dbdb/4b4b/4b4b\x1b\\");   // red:     #db4b4b
            p.process(b"\x1b]4;2;rgb:9d9d/cccc/6565\x1b\\");   // green:   #9dcc65
            p.process(b"\x1b]4;3;rgb:ecec/bfbf/7f7f\x1b\\");   // yellow:  #ecbf7f
            p.process(b"\x1b]4;4;rgb:7a7a/aaaa/dada\x1b\\");   // blue:    #7aaada
            p.process(b"\x1b]4;5;rgb:b2b2/8c8c/eded\x1b\\");   // magenta: #b28ced
            p.process(b"\x1b]4;6;rgb:5454/c8c8/aaaa\x1b\\");   // cyan:    #54c8aa
            p.process(b"\x1b]4;7;rgb:c0c0/caca/f5f5\x1b\\");   // white:   #c0caf5
            // Bright colors
            p.process(b"\x1b]4;8;rgb:5454/5454/6666\x1b\\");   // brblack: #545466
            p.process(b"\x1b]4;9;rgb:ffff/7575/7575\x1b\\");   // brred:   #ff7575
            p.process(b"\x1b]4;10;rgb:c3c3/e8e8/8888\x1b\\");  // brgreen: #c3e888
            p.process(b"\x1b]4;11;rgb:ffff/cccc/9999\x1b\\");  // bryellow:#ffcc99
            p.process(b"\x1b]4;12;rgb:7a7a/bbbb/dada\x1b\\");  // brblue:  #7abbda
            p.process(b"\x1b]4;13;rgb:bbbb/9c9c/f0f0\x1b\\");  // brmagenta:#bb9cf0
            p.process(b"\x1b]4;14;rgb:6c6c/dcdc/bfbf\x1b\\");  // brcyan:  #6cdcbf
            p.process(b"\x1b]4;15;rgb:d4d4/d5d5/e0e0\x1b\\");  // brwhite: #d4d5e0
        }

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
