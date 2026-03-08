use std::process::Command;

pub struct Git;

impl Git {
    /// Check if the current directory is inside a git repository
    pub fn is_git_repo() -> bool {
        Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Get list of modified files that aren't gitignored
    pub fn get_modified_files() -> Vec<String> {
        let output = Command::new("git")
            .args(["status", "--porcelain", "--untracked-files=all"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|line| {
                        // Parse git status output: XY filename or XY "filename"
                        // Skip the status codes (first 2 chars + space)
                        if line.len() > 3 {
                            line[3..].trim_matches('"').to_string()
                        } else {
                            line.to_string()
                        }
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    /// Check if there are any changes to commit
    pub fn has_changes() -> bool {
        !Self::get_modified_files().is_empty()
    }

    /// Stage all changes and commit with the given message
    pub fn commit_all(message: &str) -> Result<(), String> {
        // First, add all changes
        let add_output = Command::new("git")
            .args(["add", "-A"])
            .output()
            .map_err(|e| format!("Failed to run git add: {}", e))?;

        if !add_output.status.success() {
            let stderr = String::from_utf8_lossy(&add_output.stderr);
            return Err(format!("git add failed: {}", stderr));
        }

        // Then commit
        let commit_output = Command::new("git")
            .args(["commit", "-m", message])
            .output()
            .map_err(|e| format!("Failed to run git commit: {}", e))?;

        if !commit_output.status.success() {
            let stderr = String::from_utf8_lossy(&commit_output.stderr);
            return Err(format!("git commit failed: {}", stderr));
        }

        Ok(())
    }
}
