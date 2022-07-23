use super::*;
use std::process::{Child, Command};
use std::time::Duration;
use wait_timeout::ChildExt;

/// Qemu run configuration
#[derive(Debug, Clone)]
pub struct QemuConfig {
    pub qemu_path: String,
    pub bios_path: String,
    pub vars_path: String,
    pub shell_path: String,
    pub output_path: String,
    pub drives: Vec<QemuDriveConfig>,
    pub additional_args: Vec<String>,
}

impl Default for QemuConfig {
    fn default() -> Self {
        Self {
            qemu_path: "qemu-system-x86_64".to_string(),
            bios_path: "OVMF.fd".to_string(),
            vars_path: "OVMF_VARS.fd".to_string(),
            shell_path: "UefiShell.iso".to_string(),
            output_path: "Output.txt".to_string(),
            drives: Vec::new(),
            additional_args: vec![],
        }
    }
}

impl QemuConfig {
    /// Run an instance of qemu with the given config
    pub fn run(&self) -> Result<QemuProcess> {
        let mut args = Vec::new();
        let mut index = 0;

        args.push("-drive".to_string());
        args.push(format!(
            "if=pflash,format=raw,file={},index={}",
            self.bios_path, index,
        ));
        index += 1;

        args.push("-drive".to_string());
        args.push(format!(
            "if=pflash,format=raw,file={},index={}",
            self.vars_path, index,
        ));
        index += 1;

        args.push("-drive".to_string());
        args.push(format!(
            "format=raw,file={},index={}",
            self.shell_path, index,
        ));
        index += 1;

        for drive in self.drives.iter() {
            args.push("-drive".to_string());
            args.push(format!(
                "file={},index={},media={},format={}",
                drive.file, index, drive.media, drive.format
            ));
            index += 1;
        }

        args.push("-serial".to_string());
        args.push(format!("file:{}", self.output_path));

        args.extend(self.additional_args.iter().cloned());

        let child = Command::new(&self.qemu_path).args(args).spawn()?;
        Ok(QemuProcess { child })
    }
}

/// Qemu drive configuration
#[derive(Debug, Clone)]
pub struct QemuDriveConfig {
    pub file: String,
    pub media: String,
    pub format: String,
}

impl QemuDriveConfig {
    pub fn new(file: &str, media: &str, format: &str) -> Self {
        Self {
            file: file.to_string(),
            media: media.to_string(),
            format: format.to_string(),
        }
    }
}

pub struct QemuProcess {
    child: Child,
}

impl QemuProcess {
    /// Wait for the process to exit for `duration`.
    ///
    /// Returns `true` if the process exited and false if the timeout expired.
    pub fn wait(&mut self, duration: Duration) -> Option<i32> {
        self.child
            .wait_timeout(duration)
            .expect("Failed to wait on child process")
            .map(|exit_status| exit_status.code().unwrap_or(0))
    }

    /// Kill the process.
    pub fn kill(&mut self) -> std::io::Result<()> {
        self.child.kill()
    }
}
