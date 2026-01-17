use std::process::Command;
use anyhow::Result;

pub fn capture_diff() -> Result<String> {
    // Check if we're in a git repository
    let status = Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .output()?;
    
    if !status.status.success() {
        return Ok(String::new()); // Not a git repository
    }
    
    // Capture working tree diff
    let output = Command::new("git")
        .args(&["diff", "--no-color"])
        .output()?;
    
    if !output.status.success() {
        anyhow::bail!("Failed to capture git diff");
    }
    
    Ok(String::from_utf8(output.stdout)?)
}

pub fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(&["branch", "--show-current"])
        .output()?;
    
    if !output.status.success() {
        return Ok("unknown".to_string());
    }
    
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

pub fn get_repo_root() -> Result<String> {
    let output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()?;
    
    if !output.status.success() {
        anyhow::bail!("Not in a git repository");
    }
    
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_diff() {
        // This test will only pass in a git repository
        let result = capture_diff();
        assert!(result.is_ok());
    }
}
