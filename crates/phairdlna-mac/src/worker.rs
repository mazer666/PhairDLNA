//! The thread that talks to the administration surface.
//!
//! What: the thread that owns the client, the commands it takes and the
//!       answers it gives.
//! Why: the main thread of a menu bar application belongs to the drawing and
//!      must never wait for the network; a request that blocks the event loop
//!      makes the menu stutter and freezes the whole system (`MB-002`).
//! Example: the application asks the worker for the state of the server and
//!          takes the answer when the timer of the menu bar ticks.
//!
//! **One request at a time.** The worker takes one command at a time and the
//! channel of the commands holds one command, so that the application never
//! has more than one request in flight.
//!
//! **The worker owns the client.** The client of the administration surface
//! lives in the worker and not in the application, because the client opens a
//! socket and a socket belongs to the thread that waits for it.

use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError, TrySendError};
use std::thread::JoinHandle;

use crate::ipc::client::{AdminClient, ClientError};
use crate::ipc::http::Transport;
use crate::ipc::snapshots::{RescanReport, StatusSnapshot, TransferSnapshot};
use crate::model::{RootInfo, ServerSnapshot, ServerState, TransferInfo};

/// What the application asks the worker to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Read the state of the server again.
    Refresh,
    /// Read the library again.
    Rescan,
    /// Fetch the full diagnostics JSON report.
    ExportDiagnostics,
}

/// What the worker answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// The state of the server.
    State(Box<ServerSnapshot>),
    /// What the scan of the library found.
    Scanned(RescanReport),
    /// The raw diagnostics JSON report.
    Diagnostics(String),
    /// The surface could not be reached.
    Failed(ClientError),
}

/// Why the worker could not be started.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerError {
    /// What the system said.
    pub reason: String,
}

/// The thread that talks to the administration surface.
#[derive(Debug)]
pub struct Worker {
    /// The way the application asks for something.
    commands: Option<SyncSender<Command>>,
    /// The way the worker answers.
    answers: Option<Receiver<Answer>>,
    /// The thread itself, so that it is joined when the worker is dropped.
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    /// Starts the worker and asks it for the state of the server.
    ///
    /// The answer is an error when the system refuses to start the thread;
    /// the application then says so and does not draw a menu of nothing.
    pub fn spawn<T: Transport + Send + 'static>(
        client: AdminClient<T>,
    ) -> Result<Self, WorkerError> {
        let (sender, receiver) = sync_channel(1);
        let (answers_out, answers) = sync_channel(1);
        sender.send(Command::Refresh).map_err(|error| WorkerError {
            reason: error.to_string(),
        })?;
        let thread = std::thread::Builder::new()
            .name("phairdlna-admin".to_owned())
            .spawn(move || {
                let mut client = client;
                run(&mut client, receiver, answers_out);
            })
            .map_err(|error| WorkerError {
                reason: error.to_string(),
            })?;
        Ok(Self {
            commands: Some(sender),
            answers: Some(answers),
            thread: Some(thread),
        })
    }

    /// Asks the worker for something.
    ///
    /// The answer is `false` when the worker is busy: the request is dropped
    /// and not queued, so that the application never holds more than one
    /// request in flight.
    pub fn ask(&self, command: Command) -> bool {
        let Some(commands) = &self.commands else {
            return false;
        };
        match commands.try_send(command) {
            Ok(()) => true,
            Err(TrySendError::Full(_)) => false,
            Err(TrySendError::Disconnected(_)) => false,
        }
    }

    /// Takes an answer of the worker, if there is one.
    pub fn take(&self) -> Option<Answer> {
        let answers = self.answers.as_ref()?;
        match answers.try_recv() {
            Ok(answer) => Some(answer),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }
}

impl Drop for Worker {
    /// Stops the thread when the worker is dropped.
    ///
    /// The channel of the commands is closed first, so that the loop of the
    /// thread ends. The channel of the answers is closed as well: a thread
    /// that waits for room in that channel would never end, and the join
    /// would wait for it forever. The join then waits for the thread, and
    /// the process does not end while a request is in flight.
    fn drop(&mut self) {
        self.commands = None;
        self.answers = None;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// The loop of the thread.
fn run<T: Transport>(
    client: &mut AdminClient<T>,
    commands: Receiver<Command>,
    answers: SyncSender<Answer>,
) {
    while let Ok(command) = commands.recv() {
        let answer = answer_of(client, command);
        if answers.send(answer).is_err() {
            break;
        }
    }
}

/// The answer of one command.
fn answer_of<T: Transport>(client: &mut AdminClient<T>, command: Command) -> Answer {
    match command {
        Command::Refresh => match client.status() {
            Ok(status) => Answer::State(Box::new(state_of(&status, client))),
            Err(error) => Answer::Failed(error),
        },
        Command::Rescan => match client.rescan() {
            Ok(report) => Answer::Scanned(report),
            Err(error) => Answer::Failed(error),
        },
        Command::ExportDiagnostics => match client.raw_diagnostics() {
            Ok(json) => Answer::Diagnostics(json),
            Err(error) => Answer::Failed(error),
        },
    }
}

/// The state of the server, out of the answer of the surface.
fn state_of<T: Transport>(status: &StatusSnapshot, client: &mut AdminClient<T>) -> ServerSnapshot {
    let mut snapshot = ServerSnapshot {
        state: ServerState::Running,
        device_name: status.device_name.clone(),
        device_address: status.device_address.clone(),
        discovery_available: status.discovery_available,
        uptime_seconds: status.uptime_seconds,
        active_streams: status.active_streams,
        catalogue_objects: status.catalogue_objects,
        roots: status.roots.iter().map(root_of).collect(),
        effective_interface: status.effective_interface.clone(),
        effective_address: status.effective_address.clone(),
        listener_state: status.listener_state.clone(),
        last_recovery_event: status.last_recovery_event.clone(),
        ..ServerSnapshot::default()
    };
    if let Ok(diagnostics) = client.diagnostics() {
        snapshot.warnings = diagnostics.warnings;
        snapshot.transfers = diagnostics.transfers.iter().map(transfer_of).collect();
        snapshot.last_scan = diagnostics.last_scan;
    }
    snapshot
}

/// One root of the library, as the menu shows it.
fn root_of(root: &crate::ipc::snapshots::RootSnapshot) -> RootInfo {
    RootInfo {
        name: root.name.clone(),
        mounted: root.mounted,
        objects: root.objects,
    }
}

/// One stream, as the menu shows it.
fn transfer_of(transfer: &TransferSnapshot) -> TransferInfo {
    TransferInfo {
        client: transfer.client.clone(),
        title: transfer.title.clone(),
    }
}
