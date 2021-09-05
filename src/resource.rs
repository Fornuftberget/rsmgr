use crate::cli;
use std::convert::TryInto;

pub fn update(res: &String, cfg: &cli::Config) -> Result<String, git2::Error> {
    let (name, dest) = res.split_once(':').unwrap();
    let mut callbacks = git2::RemoteCallbacks::new();

    match &cfg.ssh_key {
        Some(path) => {
            if path.is_file() {
                callbacks.credentials(|_url, username_from_url, _allowed_types| {
                    git2::Cred::ssh_key(
                        username_from_url.unwrap(),
                        None,
                        cfg.ssh_key.as_ref().unwrap().as_path(),
                        None,
                    )
                });
            } else if path.is_dir() {
                // if path is a dir, look for <path>/id_rsa.<name>
                // Poor mans agent? :D
                callbacks.credentials(|_url, username_from_url, _allowed_types| {
                    let mut p = cfg.ssh_key.clone().unwrap();
                    p.push(format!("id_rsa.{}", name));
                    git2::Cred::ssh_key(username_from_url.unwrap(), None, p.as_path(), None)
                });
            }
        }
        None => {
            callbacks.credentials(|_url, username_from_url, _allowed_types| {
                git2::Cred::ssh_key_from_agent(username_from_url.unwrap())
            });
        }
    }

    let mut last_sec: u128 = 0;
    let start = std::time::SystemTime::now();
    callbacks.transfer_progress(|progress| {
        let diff = std::time::SystemTime::now().duration_since(start);
        let sec = diff.unwrap().as_millis() / 1000;
        if sec > 20 && sec != last_sec {
            last_sec = sec;
            if last_sec % 5 == 0 && progress.total_objects() > progress.received_objects() {
                let proc = progress.received_objects() as f64 / progress.total_objects() as f64;
                println!(
                    "Receiving objects {} {}/{} {:.1}% {} seconds elapsed",
                    name,
                    progress.received_objects(),
                    progress.total_objects(),
                    proc * 100.0,
                    last_sec
                );
            }
        }
        true
    });

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    // @TODO Switch to proper URL library
    let src = format!("{}{}.git", cfg.base_url, name);
    let mut dst = cfg.base_path.clone();
    dst.push(dest);

    match git2::Repository::open(&dst) {
        Ok(repo) => {
            if repo.state() != git2::RepositoryState::Clean {
                return Err(git2::Error::new(
                    git2::ErrorCode::GenericError,
                    git2::ErrorClass::Repository,
                    "repository is not in a clean state",
                ));
            }

            if cfg.discard {
                // Discard ANY local changes
                repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .use_theirs(true)
                        .remove_untracked(true)
                        .remove_ignored(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?
            }

            if let Ok(statuses) = repo.statuses(None) {
                for s in statuses.iter() {
                    if s.status() != git2::Status::WT_NEW {
                        return Err(git2::Error::new(
                            git2::ErrorCode::GenericError,
                            git2::ErrorClass::Repository,
                            "uncommitted changes, ignoring",
                        ));
                    }
                }
            }

            let repo_head = repo.head()?;
            let head_commit = repo.reference_to_annotated_commit(&repo_head)?;
            let branch = repo_head.shorthand().unwrap();

            match repo.find_remote("origin") {
                Ok(mut r) => r.fetch(&[branch], Some(&mut fetch_options), None)?,
                Err(e) => {
                    return Err(e);
                }
            };

            let fetch_head = repo.find_reference("FETCH_HEAD")?;
            let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

            let analysis = repo.merge_analysis(&[&fetch_commit])?;

            if analysis.0.is_up_to_date() {
                return Ok(String::new());
            }

            let refname = format!("refs/heads/{}", branch);
            match repo.find_reference(&refname) {
                Ok(rref) => {
                    if analysis.0.is_fast_forward() {
                        match git_rebase(&repo, &fetch_commit) {
                            Ok(_) => {
                                let rname = match rref.name() {
                                    Some(s) => s.to_string(),
                                    None => String::from_utf8_lossy(rref.name_bytes()).to_string(),
                                };

                                let mut buf = String::new();

                                // Fast forward
                                buf.push_str(
                                    format!(
                                        "Fast-forward {}, setting {} to {}\n",
                                        name,
                                        rname,
                                        head_commit.id()
                                    )
                                    .as_str(),
                                );

                                //r.set_target(fetch_commit.id(), &msg).unwrap();
                                repo.set_head(&rname)?;

                                git_log(&repo, &head_commit, &mut buf)?;

                                return Ok(buf);
                            }
                            Err(e) => return Err(e),
                        }
                    } else if analysis.0.is_normal() {
                        git_merge(&repo, &head_commit, &fetch_commit)?
                    }
                }
                Err(_) => {
                    repo.reference(
                        &refname,
                        fetch_commit.id(),
                        true,
                        &format!("Setting {}@{} to {}", name, branch, fetch_commit.id()),
                    )?;

                    repo.set_head(&refname)?;
                    repo.checkout_head(Some(
                        git2::build::CheckoutBuilder::default()
                            .allow_conflicts(true)
                            .conflict_style_merge(true)
                            .force(),
                    ))?
                }
            };
        }
        Err(_e) => {
            let mut builder = git2::build::RepoBuilder::new();

            builder.fetch_options(fetch_options);

            let start = std::time::SystemTime::now();
            match builder.clone(&src, dst.as_path()) {
                Ok(v) => {
                    let end = std::time::SystemTime::now();
                    let diff = end.duration_since(start);
                    return Ok(format!(
                        "Cloned {} @ {} in {} seconds",
                        name,
                        v.head().unwrap().shorthand().unwrap(),
                        diff.unwrap().as_secs(),
                    ));
                }
                Err(e) => return Err(e),
            }
        }
    };
    Ok(String::new())
}

fn git_log(
    repo: &git2::Repository,
    head_commit: &git2::AnnotatedCommit,
    buf: &mut String,
) -> Result<(), git2::Error> {
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::NONE)?;
    revwalk.push_head()?;

    for id in revwalk {
        if let Ok(commit) = repo.find_commit(id.unwrap()) {
            let time = commit.time();
            let dt = chrono::NaiveDateTime::from_timestamp(
                time.seconds(),
                time.offset_minutes().try_into().unwrap(),
            );
            buf.push_str(
                format!(
                    "\t{} {:.8}: {}\n",
                    dt.format("%Y-%m-%d"),
                    commit.id(),
                    commit.summary().unwrap()
                )
                .as_str(),
            );
            if commit.id() == head_commit.id() {
                break;
            }
        }
    }

    Ok(())
}

fn git_merge(
    repo: &git2::Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        repo.checkout_index(Some(&mut idx), None)?;
        return Err(git2::Error::new(
            git2::ErrorCode::MergeConflict,
            git2::ErrorClass::Merge,
            "merge conflict detected",
        ));
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;

    // now create the merge commit
    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;

    // Do our merge commit and set current branch head to that commit.
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;

    // Set working tree to match head.
    repo.checkout_head(None)?;
    Ok(())
}

fn git_rebase(repo: &git2::Repository, remote: &git2::AnnotatedCommit) -> Result<(), git2::Error> {
    let mut rebase: git2::Rebase = repo.rebase(
        None,
        None,
        Some(remote),
        Some(&mut git2::RebaseOptions::default()),
    )?;

    for result in rebase.next() {
        if let Err(e) = result {
            return Err(e);
        }
    }

    let sig = repo.signature()?;
    rebase.finish(Some(&sig))
}
