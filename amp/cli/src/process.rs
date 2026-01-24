use tokio::process::{Child, ChildStderr, ChildStdout, Command};
use std::process::Stdio;
use anyhow::Result;

pub struct AgentProcess {
    child: Child,
    command: String,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
}

impl AgentProcess {
    pub async fn spawn(command: &str, capture_output: bool) -> Result<Self> {
        tracing::info!("Spawning agent process: {}", command);
        
        let mut cmd = if cfg!(target_os = "windows") {
            // On Windows, use PowerShell to inherit the full environment
            let mut cmd = Command::new("powershell");
            cmd.args(&["-Command", command]);
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(command);
            cmd
        };
        
        // Inherit the current environment
        cmd.env_clear();
        for (key, value) in std::env::vars() {
            cmd.env(key, value);
        }
        
        // Allow full terminal interaction for interactive agents
        cmd.stdin(Stdio::inherit());
        if capture_output {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
        } else {
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());
        }
        
        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        
        Ok(Self {
            child,
            command: command.to_string(),
            stdout,
            stderr,
        })
    }

    pub async fn wait_for_completion(&mut self) -> Result<i32> {
        let status = self.child.wait().await?;
        let exit_code = status.code().unwrap_or(-1);
        
        tracing::info!("Agent process '{}' completed with exit code: {}", self.command, exit_code);
        Ok(exit_code)
    }

    pub async fn kill(&mut self) -> Result<()> {
        tracing::info!("Killing agent process: {}", self.command);
        self.child.kill().await?;
        Ok(())
    }

    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }

    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.stdout.take()
    }

    pub fn take_stderr(&mut self) -> Option<ChildStderr> {
        self.stderr.take()
    }
}

impl Drop for AgentProcess {
    fn drop(&mut self) {
        if let Some(_) = self.child.id() {
            tracing::debug!("Cleaning up agent process: {}", self.command);
            let _ = self.child.start_kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_spawn() {
        let mut process = AgentProcess::spawn("echo 'hello world'", false).await.unwrap();
        let exit_code = process.wait_for_completion().await.unwrap();
        assert_eq!(exit_code, 0);
    }

    #[tokio::test]
    async fn test_process_kill() {
        let mut process = AgentProcess::spawn("sleep 10", false).await.unwrap();
        process.kill().await.unwrap();
    }
}
