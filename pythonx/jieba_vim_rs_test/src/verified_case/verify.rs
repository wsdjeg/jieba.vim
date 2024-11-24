use super::cases::VerifiableCase;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn write_group_vader(path: &Path, sub_vader_paths: &[PathBuf]) {
    let mut f = File::create(path).unwrap();
    for path in sub_vader_paths {
        writeln!(f, "Include: {}", path.to_str().unwrap()).unwrap();
    }
}

/// Verify all cases in the given group. Return `Err(log)` if verification
/// fails.
pub fn verify_cases<C>(
    group_name: String,
    cases: HashMap<String, Vec<C>>,
) -> Result<(), String>
where
    C: VerifiableCase + PartialEq + Serialize + DeserializeOwned,
{
    let basedir: PathBuf = [
        env::var("CARGO_MANIFEST_DIR").unwrap(),
        ".verified_cases".into(),
    ]
    .iter()
    .collect();
    fs::create_dir(&basedir).ok();

    // Create the group directory if not exists.
    fs::create_dir(basedir.join(&group_name)).ok();

    // Try loading verification results, and record the indices of the verified
    // cases.
    let mut verified_indices: HashMap<String, Vec<usize>> = HashMap::new();
    for (case_name, sub_cases) in cases.iter() {
        // Whether each case has been verified.
        let ind = verified_indices.entry(case_name.to_string()).or_default();
        for (i, case) in sub_cases.iter().enumerate() {
            let verified_case_path = basedir.join(format!(
                "{}/{}-{}-verified.json",
                group_name,
                case_name,
                i + 1,
            ));
            if let Ok(s) = fs::read_to_string(verified_case_path) {
                let verified_case: C = serde_json::from_str(&s).unwrap();
                if case == &verified_case {
                    ind.push(i);
                }
            }
        }
    }

    // Create a minimal vimrc if not already exists.
    let vimrc_path = basedir.join("vimrc");
    if let Ok(mut file) = File::create_new(vimrc_path) {
        write!(file, "set rtp+=~/.vim/bundle/vader.vim\n").unwrap();
    }

    // Create the vim vader files for cases that are not verified.
    let mut case_paths = Vec::new();
    for (case_name, sub_cases) in cases.iter() {
        let ind = verified_indices.get(case_name).unwrap();
        for (i, case) in sub_cases
            .iter()
            .enumerate()
            .filter(|(i, _)| !ind.contains(i))
        {
            let case_path = basedir.join(format!(
                "{}/{}-{}.vader",
                group_name,
                case_name,
                i + 1
            ));
            case.to_vader(&case_path);
            case_paths.push(case_path);
        }
    }
    // Create the group vader file.
    let group_path = basedir.join(format!("{}.vader", group_name));
    write_group_vader(&group_path, &case_paths);

    // Run the tests.
    let proc_out = Command::new("vim")
        .args(&[
            "-N",
            "-u",
            "vimrc",
            &format!("+:Vader! {}", group_path.to_str().unwrap()),
        ])
        .current_dir(&basedir)
        .output()
        .unwrap();
    if proc_out.status.success() {
        // Write cache to disk to indicate verification success.
        for (case_name, sub_cases) in cases.iter() {
            let ind = verified_indices.get(case_name).unwrap();
            for (i, case) in sub_cases
                .iter()
                .enumerate()
                .filter(|(i, _)| !ind.contains(i))
            {
                let verified_case_path = basedir.join(format!(
                    "{}/{}-{}-verified.json",
                    group_name,
                    case_name,
                    i + 1,
                ));
                let s = serde_json::to_string(case).unwrap();
                let mut file = File::create(verified_case_path).unwrap();
                write!(file, "{}", s).unwrap();
            }
        }
        Ok(())
    } else {
        // Otherwise, return the stderr of the process.
        let stderr = String::from_utf8_lossy(&proc_out.stderr);
        Err(stderr.into())
    }
}
