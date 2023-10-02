//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_core::QEventLoop;

use anyhow::Error;
use crossbeam::channel::{Receiver, Sender, unbounded};

use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};

use rpfm_lib::integrations::{log::info, git::GitResponse};

use crate::mod_manager::{game_config::GameConfig, mods::ShareableMod};
use crate::updater::APIResponse;

/// This const is the standard message in case of message communication error. If this happens, crash the program.
pub const THREADS_COMMUNICATION_ERROR: &str = "Error in thread communication system. Response received: ";
pub const THREADS_SENDER_ERROR: &str = "Error in thread communication system. Sender failed to send message.";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the senders and receivers necessary to communicate both, backend and frontend threads.
///
/// You can use them by using the send/recv functions implemented for it.
pub struct CentralCommand<T: Send + Sync + Debug> {
    sender_background: Sender<(Sender<T>, Command)>,
    sender_network: Sender<(Sender<T>, Command)>,

    receiver_background: Receiver<(Sender<T>, Command)>,
    receiver_network: Receiver<(Sender<T>, Command)>,

    try_lock: AtomicBool,
}

/// This enum defines the commands (messages) you can send to the background thread in order to execute actions.
///
/// Each command should include the data needed for his own execution. For a more detailed explanation, check the
/// docs of each command.
#[derive(Debug)]
pub enum Command {
    Exit,
    CheckUpdates,
    UpdateMainProgram,
    CheckSchemaUpdates,
    UpdateSchemas(String),
    CheckTranslationsUpdates,
    UpdateTranslations,
    GetStringFromLoadOrder(GameConfig),
    GetLoadOrderFromString(String),
}

/// This enum defines the responses (messages) you can send to the to the UI thread as result of a command.
///
/// Each response should be named after the types of the items it carries.
#[derive(Debug)]
pub enum Response {
    Success,
    Error(Error),
    String(String),
    APIResponse(APIResponse),
    APIResponseGit(GitResponse),
    VecShareableMods(Vec<ShareableMod>),
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Default implementation of `CentralCommand`.
impl<T: Send + Sync + Debug> Default for CentralCommand<T> {
    fn default() -> Self {
        let (sender_background, receiver_background) = unbounded();
        let (sender_network, receiver_network) = unbounded();
        let try_lock = AtomicBool::new(false);
        Self {
            sender_background,
            sender_network,
            receiver_background,
            receiver_network,
            try_lock,
        }
    }
}

/// Implementation of `CentralCommand`.
impl<T: Send + Sync + Debug> CentralCommand<T> {

    /// This function serves as a generic way for commands to be sent to the backend.
    ///
    /// It returns the receiver which will receive the answers for the command, if any.
    fn send<S: Send + Sync + Debug>(sender: &Sender<(Sender<T>, S)>, data: S) -> Receiver<T> {
        let (sender_back, receiver_back) = unbounded();
        if let Err(error) = sender.send((sender_back, data)) {
            panic!("{THREADS_SENDER_ERROR}: {error}");
        }

        receiver_back
    }

    /// This function serves to send a message from the main thread to the background thread.
    ///
    /// It returns the receiver which will receive the answers for the command, if any.
    pub fn send_background(&self, data: Command) -> Receiver<T> {
        Self::send(&self.sender_background, data)
    }

    /// This function serves to send a message from the main thread to the network thread.
    ///
    /// It returns the receiver which will receive the answers for the command, if any.
    pub fn send_network(&self, data: Command) -> Receiver<T> {
        Self::send(&self.sender_network, data)
    }

    /// This function serves to send a message back through a generated channel.
    pub fn send_back(sender: &Sender<T>, data: T) {
        if let Err(error) = sender.send(data) {
            panic!("{THREADS_SENDER_ERROR}: {error}");
        }
    }

    /// This functions serves to receive messages on the background thread.
    ///
    /// This function does only try once, and it locks the thread. Panics if the response fails.
    pub fn recv_background(&self) -> (Sender<T>, Command) {
        let response = self.receiver_background.recv();
        match response {
            Ok(data) => data,
            Err(_) => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}")
        }
    }

    /// This functions serves to receive messages on the network thread.
    ///
    /// This function does only try once, and it locks the thread. Panics if the response fails.
    pub fn recv_network(&self) -> (Sender<T>, Command) {
        let response = self.receiver_network.recv();
        match response {
            Ok(data) => data,
            Err(_) => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}")
        }
    }

    /// This functions serves to receive messages from a generated channel.
    ///
    /// This function does only try once, and it locks the thread. Panics if the response fails.
    pub fn recv(receiver: &Receiver<T>) -> T {
        let response = receiver.recv();
        match response {
            Ok(data) => data,
            Err(_) => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}")
        }
    }

    /// This functions serves to receive messages from a generated channel.
    ///
    /// This function will keep asking for a response, keeping the UI responsive. Use it for heavy tasks.
    ///
    /// NOTE: Beware of other events triggering when this keeps the UI enabled. It can lead to crashes.
    pub fn recv_try(&self, receiver: &Receiver<T>) -> T {
        let event_loop = unsafe { QEventLoop::new_0a() };

        // Lock this function after the first execution, until it gets freed again.
        if !self.try_lock.load(Ordering::SeqCst) {
            self.try_lock.store(true, Ordering::SeqCst);

            loop {

                // Check the response and, in case of error, try again. If the error is "Disconnected", CTD.
                let response = receiver.try_recv();
                match response {
                    Ok(data) => {
                        self.try_lock.store(false, Ordering::SeqCst);
                        return data
                    },
                    Err(error) => if error.is_disconnected() {
                        panic!("{THREADS_COMMUNICATION_ERROR}{response:?}")
                    }
                }
                unsafe { event_loop.process_events_0a(); }
            }
        }

        // If we're locked due to another execution, use recv instead.
        else {
            info!("Race condition avoided? Two items calling recv_try on the same execution crashes.");
            Self::recv(receiver)
        }
    }
}
