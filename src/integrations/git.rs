use anyhow::Result;
use git2::{Repository, Commit, Oid, BranchType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    pub repository_url: Option<String>,
    pub current_branch: String,
    pub total_commits: usize,
    pub contributors: Vec<Contributor>,
    pub recent_commits: Vec<CommitInfo>,
    pub branches: Vec<String>,
    pub file_changes: HashMap<String, FileChangeStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contributor {
    pub name: String,
    pub email: String,
    pub commit_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: DateTime<Utc>,
    pub files_changed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeStats {
    pub additions: usize,
    pub deletions: usize,
    pub commits: usize,
}

pub struct GitAnalyzer;

impl GitAnalyzer {
    pub fn analyze_repository(path: &str) -> Result<GitInfo> {
        let repo = Repository::open(path)?;
        
        let current_branch = Self::get_current_branch(&repo)?;
        let commits = Self::analyze_commits(&repo)?;
        let contributors = Self::analyze_contributors(&repo)?;
        let branches = Self::get_branches(&repo)?;
        let repository_url = Self::get_repository_url(&repo);
        let file_changes = Self::analyze_file_changes(&repo)?;

        Ok(GitInfo {
            repository_url,
            current_branch,
            total_commits: commits.len(),
            contributors,
            recent_commits: commits.into_iter().take(10).collect(),
            branches,
            file_changes,
        })
    }

    fn get_current_branch(repo: &Repository) -> Result<String> {
        let head = repo.head()?;
        if let Some(name) = head.shorthand() {
            Ok(name.to_string())
        } else {
            Ok("HEAD".to_string())
        }
    }

    fn analyze_commits(repo: &Repository) -> Result<Vec<CommitInfo>> {
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut commits = Vec::new();

        for oid in revwalk.take(100) {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            
            let commit_info = CommitInfo {
                hash: oid.to_string()[..8].to_string(),
                message: commit.message().unwrap_or("No message").lines().next().unwrap_or("").to_string(),
                author: format!("{}", commit.author().name().unwrap_or("Unknown")),
                date: DateTime::from_timestamp(commit.time().seconds(), 0)
                    .unwrap_or_else(|| Utc::now()),
                files_changed: Self::count_changed_files(&repo, &commit),
            };

            commits.push(commit_info);
        }

        Ok(commits)
    }

    fn analyze_contributors(repo: &Repository) -> Result<Vec<Contributor>> {
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        let mut contributor_counts: HashMap<String, (String, usize)> = HashMap::new();

        for oid in revwalk {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            let name = commit.author().name().unwrap_or("Unknown").to_string();
            let email = commit.author().email().unwrap_or("unknown@example.com").to_string();
            
            let entry = contributor_counts.entry(email.clone()).or_insert((name.clone(), 0));
            entry.1 += 1;
        }

        let mut contributors: Vec<Contributor> = contributor_counts
            .into_iter()
            .map(|(email, (name, count))| Contributor {
                name,
                email,
                commit_count: count,
            })
            .collect();

        contributors.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));
        Ok(contributors)
    }

    fn get_branches(repo: &Repository) -> Result<Vec<String>> {
        let branches = repo.branches(Some(BranchType::Local))?;
        let mut branch_names = Vec::new();

        for branch in branches {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                branch_names.push(name.to_string());
            }
        }

        Ok(branch_names)
    }

    fn get_repository_url(repo: &Repository) -> Option<String> {
        if let Ok(remotes) = repo.remotes() {
            if let Some(origin) = remotes.get(0) {
                if let Ok(remote) = repo.find_remote(origin) {
                    return remote.url().map(|s| s.to_string());
                }
            }
        }
        None
    }

    fn analyze_file_changes(repo: &Repository) -> Result<HashMap<String, FileChangeStats>> {
        let mut file_stats: HashMap<String, FileChangeStats> = HashMap::new();
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        for oid in revwalk.take(100) {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            
            if commit.parent_count() > 0 {
                let parent = commit.parent(0)?;
                let diff = repo.diff_tree_to_tree(Some(&parent.tree()?), Some(&commit.tree()?), None)?;
                
                diff.foreach(
                    &mut |delta, _| {
                        if let Some(path) = delta.new_file().path() {
                            if let Some(path_str) = path.to_str() {
                                let stats = file_stats.entry(path_str.to_string()).or_insert(FileChangeStats {
                                    additions: 0,
                                    deletions: 0,
                                    commits: 0,
                                });
                                stats.commits += 1;
                            }
                        }
                        true
                    },
                    None,
                    None,
                    None,
                )?;
            }
        }

        Ok(file_stats)
    }

    fn count_changed_files(repo: &Repository, commit: &Commit) -> usize {
        if commit.parent_count() == 0 {
            return 0;
        }

        if let Ok(parent) = commit.parent(0) {
            if let (Ok(tree), Ok(parent_tree)) = (commit.tree(), parent.tree()) {
                if let Ok(diff) = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None) {
                    return diff.deltas().len();
                }
            }
        }
        
        0
    }
}