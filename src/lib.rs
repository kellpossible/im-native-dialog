//! This crate is a wrapper around [FileDialog] for use with immediate mode gui
//! libraries. See [ImNativeFileDialog] for more information.

use std::path::PathBuf;

use native_dialog::FileDialog;
use thiserror::Error;

/// Error associated with [NativeFileDialog].
#[derive(Error, Debug)]
pub enum ImNativeDialogError {
    #[error("The dialog is already open.")]
    AlreadyOpen,
}

/// A wrapper around [FileDialog] for use with immediate mode gui
/// libraries. The `show*()` methods create a [FileDialog] in a new
/// thread, and the result is returned to this object via
/// [crossbeam_channel], ready to be polled by the ui using
/// [ImNativeFileDialog::check()]
pub struct ImNativeFileDialog<T> {
    receiver: Option<crossbeam_channel::Receiver<Result<T, native_dialog::Error>>>,
}

impl<T> Default for ImNativeFileDialog<T> {
    fn default() -> Self {
        Self { receiver: None }
    }
}

impl ImNativeFileDialog<Vec<PathBuf>> {
    /// Shows a dialog that let users to open multiple files using [FileDialog::show_open_multiple_file()].
    pub fn show_open_multiple_file(
        &mut self,
        location: Option<PathBuf>,
    ) -> Result<(), ImNativeDialogError> {
        self.show(|sender, dialog| {
            let dialog = match &location {
                Some(location) => dialog.set_location(location),
                None => dialog,
            };
            let result = dialog.show_open_multiple_file();
            sender
                .send(result)
                .expect("error sending show_open_multiple_file result to ui");
            drop(location)
        })
    }
}

impl ImNativeFileDialog<Option<PathBuf>> {
    /// Shows a dialog that let users to open one directory using [FileDialog::show_open_single_dir()].
    pub fn open_single_dir(
        &mut self,
        location: Option<PathBuf>,
    ) -> Result<(), ImNativeDialogError> {
        self.show(|sender, dialog| {
            let dialog = match &location {
                Some(location) => dialog.set_location(location),
                None => dialog,
            };
            let result = dialog.show_open_single_dir();
            sender
                .send(result)
                .expect("error sending open_single_dir result to ui");
            drop(location)
        })
    }

    /// Shows a dialog that let users to open one file using [FileDialog::show_open_single_file()].
    pub fn open_single_file(
        &mut self,
        location: Option<PathBuf>,
    ) -> Result<(), ImNativeDialogError> {
        self.show(|sender, dialog| {
            let dialog = match &location {
                Some(location) => dialog.set_location(location),
                None => dialog,
            };
            let result = dialog.show_open_single_file();
            sender
                .send(result)
                .expect("error sending open_single_file result to ui");
            drop(location)
        })
    }

    /// Shows a dialog that let users to save one file using [FileDialog::show_save_single_file()].
    pub fn show_save_single_file(
        &mut self,
        location: Option<PathBuf>,
    ) -> Result<(), ImNativeDialogError> {
        self.show(|sender, dialog| {
            let dialog = match &location {
                Some(location) => dialog.set_location(location),
                None => dialog,
            };
            let result = dialog.show_save_single_file();
            sender
                .send(result)
                .expect("error sending show_save_single_file result to ui");
            drop(location)
        })
    }
}

impl<T: Send + 'static + Default> ImNativeFileDialog<T> {
    /// Show a customized version of [FileDialog], use the `run`
    /// closure to customize the dialog and show the dialog. This
    /// closure runs in its own thread.
    pub fn show<
        F: FnOnce(crossbeam_channel::Sender<Result<T, native_dialog::Error>>, FileDialog)
            + Send
            + 'static,
    >(
        &mut self,
        run: F,
    ) -> Result<(), ImNativeDialogError> {
        if self.receiver.is_some() {
            return Err(ImNativeDialogError::AlreadyOpen);
        }

        let (sender, receiver) = crossbeam_channel::bounded(1);
        std::thread::spawn(move || {
            let dialog = FileDialog::new();
            run(sender, dialog)
        });

        self.receiver = Some(receiver);

        Ok(())
    }

    /// Check if the dialog is complete. If it is complete it will
    /// return `Some` with the result of the dialog, otherwise will
    /// return `None`.
    pub fn check(&mut self) -> Option<Result<T, native_dialog::Error>> {
        match self.receiver.take() {
            Some(receiver) => match receiver.try_recv() {
                Ok(result) => Some(result),
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    log::warn!("OpenDialog channel disconnected");
                    Some(Ok(T::default()))
                }
                Err(crossbeam_channel::TryRecvError::Empty) => {
                    self.receiver = Some(receiver);
                    None
                }
            },
            None => None,
        }
    }

    /// Returns `true` if the dialog is currently open, otherwise
    /// returns `false`.
    pub fn is_open(&self) -> bool {
        self.receiver.is_some()
    }
}
