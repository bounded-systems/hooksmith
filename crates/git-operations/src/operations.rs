//! Git operations implementation for Hooksmith

use super::*;
use anyhow::Result;
use git2::{Repository, Signature};

impl GitOperationsHandler {
    /// Handle Git commit request
    pub async fn handle_git_commit(&mut self, req: GitCommitRequest) -> Result<GitOperationEvent> {
        let repo = self.open_repository()?;
        let signature = req.author
            .map(|author| Signature::now(&author.name, &author.email))
            .unwrap_or_else(|| {
                self.get_default_signature()
                    .map_err(|e| git2::Error::from_str(&e.to_string()))
            })?;
        
        // Add files to index if specified
        let files_changed = req.files.clone();
        if let Some(files) = &req.files {
            for file in files {
                let path = self.working_directory.join(file);
                repo.index()?.add_path(&path)?;
            }
        }
        
        // Create commit
        let tree_id = repo.index()?.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        
        let commit_id = repo.commit(
            Some(&"HEAD".to_string()),
            &signature,
            &signature,
            &req.message,
            &tree,
            &[],
        )?;
        
        let commit = repo.find_commit(commit_id)?;
        let branch = self.get_current_branch()?;
        
        Ok(GitOperationEvent::GitCommitResult(GitCommitResult {
            request_id: req.request_id,
            success: true,
            commit_hash: Some(commit.id().to_string()),
            files_changed,
            branch: Some(branch),
            duration_ms: None,
            error: None,
        }))
    }

    /// Handle Git push request
    pub async fn handle_git_push(&mut self, req: GitPushRequest) -> Result<GitOperationEvent> {
        let repo = self.open_repository()?;
        let remote_name = req.remote.unwrap_or_else(|| "origin".to_string());
        let branch_name = req.branch.unwrap_or_else(|| self.get_current_branch().unwrap_or_else(|_| "main".to_string()));
        
        let mut remote = repo.find_remote(&remote_name)?;
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key(
                username_from_url.unwrap_or("git"),
                None,
                std::path::Path::new(&format!("{}/.ssh/id_rsa", std::env::var("HOME").unwrap_or_default())),
                None,
            )
        });
        
        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);
        
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
        remote.push(&[&refspec], Some(&mut push_options))?;
        
        Ok(GitOperationEvent::GitPushResult(GitPushResult {
            request_id: req.request_id,
            success: true,
            output: Some(format!("Pushed to {}/{}", remote_name, branch_name)),
            branch: Some(branch_name),
            remote: Some(remote_name),
            duration_ms: None,
            error: None,
        }))
    }

    /// Handle Git pull request
    pub async fn handle_git_pull(&mut self, req: GitPullRequest) -> Result<GitOperationEvent> {
        let repo = self.open_repository()?;
        let remote_name = req.remote.unwrap_or_else(|| "origin".to_string());
        let branch_name = req.branch.unwrap_or_else(|| self.get_current_branch().unwrap_or_else(|_| "main".to_string()));
        
        let mut remote = repo.find_remote(&remote_name)?;
        remote.fetch(&[&branch_name], None, None)?;
        
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        
        let analysis = repo.merge_analysis(&[&fetch_commit])?;
        
        if analysis.0.is_up_to_date() {
            Ok(GitOperationEvent::GitPullResult(GitPullResult {
                request_id: req.request_id,
                success: true,
                output: Some("Already up to date".to_string()),
                branch: Some(branch_name),
                remote: Some(remote_name),
                duration_ms: None,
                error: None,
            }))
        } else if analysis.0.is_fast_forward() {
            let refname = format!("refs/heads/{}", branch_name);
            let mut reference = repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-forward")?;
            repo.set_head(&refname)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            
            Ok(GitOperationEvent::GitPullResult(GitPullResult {
                request_id: req.request_id,
                success: true,
                output: Some("Fast-forward merge completed".to_string()),
                branch: Some(branch_name),
                remote: Some(remote_name),
                duration_ms: None,
                error: None,
            }))
        } else {
            Err(anyhow::anyhow!("Merge required but not implemented"))
        }
    }

    /// Handle Git status request
    pub async fn handle_git_status(&mut self, req: GitStatusRequest) -> Result<GitOperationEvent> {
        let repo = self.open_repository()?;
        let mut options = git2::StatusOptions::new();
        options.include_untracked(true);
        options.include_ignored(false);
        
        let statuses = repo.statuses(Some(&mut options))?;
        let mut git_status = GitStatus {
            staged: Vec::new(),
            unstaged: Vec::new(),
            untracked: Vec::new(),
            modified: Vec::new(),
            deleted: Vec::new(),
            renamed: Vec::new(),
        };
        
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("unknown").to_string();
            let status = entry.status();
            
            if status.is_index_new() || status.is_index_modified() || status.is_index_deleted() || status.is_index_renamed() {
                git_status.staged.push(path.clone());
            }
            
            if status.is_wt_new() {
                git_status.untracked.push(path.clone());
            } else if status.is_wt_modified() {
                git_status.modified.push(path.clone());
                git_status.unstaged.push(path.clone());
            } else if status.is_wt_deleted() {
                git_status.deleted.push(path.clone());
                git_status.unstaged.push(path.clone());
            } else if status.is_wt_renamed() {
                git_status.renamed.push(path.clone());
                git_status.unstaged.push(path.clone());
            }
        }
        
        let branch = self.get_current_branch()?;
        
        Ok(GitOperationEvent::GitStatusResult(GitStatusResult {
            request_id: req.request_id,
            success: true,
            status: Some(git_status),
            branch: Some(branch),
            duration_ms: None,
            error: None,
        }))
    }

    /// Handle Git add request
    pub async fn handle_git_add(&mut self, req: GitAddRequest) -> Result<GitOperationEvent> {
        let repo = self.open_repository()?;
        let mut index = repo.index()?;
        
        for file in &req.files {
            let path = self.working_directory.join(file);
            index.add_path(&path)?;
        }
        
        index.write()?;
        
        Ok(GitOperationEvent::GitAddResult(GitAddResult {
            request_id: req.request_id,
            success: true,
            files_added: Some(req.files),
            duration_ms: None,
            error: None,
        }))
    }

    /// Handle Git note add request
    pub async fn handle_git_note_add(&mut self, req: GitNoteAddRequest) -> Result<GitOperationEvent> {
        let repo = self.open_repository()?;
        let oid = git2::Oid::from_str(&req.object)?;
        let obj = repo.find_object(oid, None)?;
        
        let message = req.message.unwrap_or_else(|| "Note added by Hooksmith".to_string());
        let signature = self.get_default_signature()?;
        
        let note_id = repo.note(&signature, &signature, None, obj.id(), &message, false)?;
        
        Ok(GitOperationEvent::GitNoteAddResult(GitNoteAddResult {
            request_id: req.request_id,
            success: true,
            note_id: Some(note_id.to_string()),
            duration_ms: None,
            error: None,
        }))
    }

    /// Handle Git note get request
    pub async fn handle_git_note_get(&mut self, req: GitNoteGetRequest) -> Result<GitOperationEvent> {
        let repo = self.open_repository()?;
        let oid = git2::Oid::from_str(&req.object)?;
        let obj = repo.find_object(oid, None)?;
        
        let note = repo.find_note(None, obj.id())?;
        let content = note.message().unwrap_or("").to_string();
        
        Ok(GitOperationEvent::GitNoteGetResult(GitNoteGetResult {
            request_id: req.request_id,
            success: true,
            note_content: Some(content),
            duration_ms: None,
            error: None,
        }))
    }
} 
