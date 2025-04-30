use std::path::PathBuf;

use clap::Parser;

pub const SEPARATORS : [char; 5] = [' ', '_', '-', '.', '/'];

/// This program renames a project directory and all occurrences of the project name in the files
/// and directories. It also renames the files and directories to match the new project name.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// New name of the project.
    /// Example: "new-project"
    #[arg(short, long)]
    name: String,
    /// Input path to the project.
    /// Example: "path/to/old-project"
    #[arg(short, long)]
    input: PathBuf,
}

fn main() {
    let args = Args::parse();
    start(args);
}

fn start(args: Args) {
    let input_path = args.input.clone();
    let input_file_name = args.input.file_name().unwrap().to_string_lossy().to_string();
    let old_name = CaseInfo::detect(&input_file_name).1;
    let new_name = CaseInfo::detect(&args.name).1;
    let output_path = args.input.parent().unwrap().join(&args.name);

    // Recursively traverse the project directory
    traverse_directory(input_path, output_path, &old_name, &new_name);
}

// Recursively traverse the directory and
// - Renames all file and directory names
// - Opens files as text and renames all occurrences of the project name
fn traverse_directory(input: PathBuf, output: PathBuf, old_name: &NormalizedName, new_name: &NormalizedName) {
    // Check if the path is a directory
    if input.is_dir() {
        // Iterate over the entries in the directory
        for entry in input.read_dir().unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let old_file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let new_file_name = transform_text(&old_file_name, old_name, new_name);
            let output_path = output.join(&new_file_name);
            
            // Create the output directory if it doesn't exist
            if !output.exists() {
                println!("Creating directory: {}", output.display());
                std::fs::create_dir_all(&output).unwrap();
            }

            traverse_directory(path, output_path, old_name, new_name);
        }
    } else {
        // If the path is a file, rename it
        rename_file(&input, &output, old_name, new_name);
    }
}

// Rename the file and all occurrences of the project name in the file
fn rename_file(input: &PathBuf, output: &PathBuf, old_name: &NormalizedName, new_name: &NormalizedName) {
    // Open the file and rename all occurrences of the project name
    if let Ok(content) =  std::fs::read_to_string(input) {
        println!("Renaming content of file: {}", input.display());
        let new_content = transform_text(&content, old_name, new_name);

        // Check if the output file exists
        if !output.exists() {
            println!("Creating file: {}", output.display());
            std::fs::write(output, new_content).unwrap();
        }
    } else {
        println!("Failed to read file, doing a simple copy: {}", input.display());
        println!("Creating file: {}", output.display());
        // Copy the file to the output directory
        std::fs::copy(&input, &output).unwrap();
    }
}

fn transform_text(input: &str, old_name: &NormalizedName, new_name: &NormalizedName) -> String {
    let all_cases = CaseInfo::all_cases();
    let mut out = input.to_string();

    for case_info in all_cases {
        let search_for = case_info.convert(old_name.clone());
        let replace_with = case_info.convert(new_name.clone());

        out = out.replace(&search_for, &replace_with);
    }

    out
}

// This struct is used to store the case information of the project name
// It contains the separator and the type of case (capitalise, upper case, lower case)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CaseInfo {
    separator: Option<char>,
    part_type: CaseType
}

// This enum is used to store the type of case (capitalise, upper case, lower case)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaseType {
    Capitalise, // My Project
    UpperCase, // MY PROJECT
    LowerCase, // my project
}

impl CaseInfo {
    fn all_cases() -> Vec<CaseInfo> {
        let mut cases = vec![];
        for part_type in [CaseType::Capitalise, CaseType::UpperCase, CaseType::LowerCase].iter() {
            cases.push(CaseInfo {
                separator: None,
                part_type: *part_type,
            });
            for separator in SEPARATORS.iter() {
                cases.push(CaseInfo {
                    separator: Some(*separator),
                    part_type: *part_type,
                });
            }
        }
        cases
    }

    fn detect(name: &str) -> (Self, NormalizedName) {
        let mut separator = None;
        for c in SEPARATORS {
            if name.contains(c) {
                separator = Some(c);
                break;
            }
        }

        let parts = if let Some(separator) = separator {
            name.split(separator).map(str::to_string).collect::<Vec<_>>()
        } else {
            vec![name.to_string()]
        };

        let part_type = if parts.iter().all(|s| s.chars().all(|c| c.is_uppercase())) {
            CaseType::UpperCase
        } else if parts.iter().all(|s| s.chars().all(|c| c.is_lowercase())) {
            CaseType::LowerCase
        } else {
            CaseType::Capitalise
        };

        (
            Self {
                separator,
                part_type,
            },
            NormalizedName {
                parts: parts.into_iter().map(|s| s.to_lowercase()).collect(),
            },
        )
    }

    fn convert(&self, normalized_name: NormalizedName) -> String {
        let separator = if let Some(separator) = self.separator {
            separator.to_string()
        } else {
            "".to_string()
        };

        normalized_name.parts.iter()
            .map(|part| match self.part_type {
                CaseType::Capitalise => part.chars().next().unwrap().to_uppercase().to_string() + &part[1..],
                CaseType::UpperCase => part.to_uppercase(),
                CaseType::LowerCase => part.to_lowercase(),
            })
            .collect::<Vec<_>>()
            .join(&separator)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedName {
    parts: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_title_case() {
        let name = "My Project";
        let (case_info, normalized_name) = CaseInfo::detect(name);
        assert_eq!(case_info.separator, Some(' '));
        assert_eq!(case_info.part_type, CaseType::Capitalise);
        assert_eq!(normalized_name.parts, vec!["my", "project"]);
    }

    #[test]
    fn test_detect_upper_case() {
        let name = "MY PROJECT";
        let (case_info, normalized_name) = CaseInfo::detect(name);
        assert_eq!(case_info.separator, Some(' '));
        assert_eq!(case_info.part_type, CaseType::UpperCase);
        assert_eq!(normalized_name.parts, vec!["my", "project"]);
    }

    #[test]
    fn test_detect_lower_case() {
        let name = "my project";
        let (case_info, normalized_name) = CaseInfo::detect(name);
        assert_eq!(case_info.separator, Some(' '));
        assert_eq!(case_info.part_type, CaseType::LowerCase);
        assert_eq!(normalized_name.parts, vec!["my", "project"]);
    }

    #[test]
    fn test_detect_snake_case() {
        let name = "my_project";
        let (case_info, normalized_name) = CaseInfo::detect(name);
        assert_eq!(case_info.separator, Some('_'));
        assert_eq!(case_info.part_type, CaseType::LowerCase);
        assert_eq!(normalized_name.parts, vec!["my", "project"]);
    }

    #[test]
    fn test_detect_kebab_case() {
        let name = "my-project";
        let (case_info, normalized_name) = CaseInfo::detect(name);
        assert_eq!(case_info.separator, Some('-'));
        assert_eq!(case_info.part_type, CaseType::LowerCase);
        assert_eq!(normalized_name.parts, vec!["my", "project"]);
    }

    #[test]
    fn test_detect_mumble_case() {
        let name = "myproject";
        let (case_info, normalized_name) = CaseInfo::detect(name);
        assert_eq!(case_info.separator, None);
        assert_eq!(case_info.part_type, CaseType::LowerCase);
        assert_eq!(normalized_name.parts, vec!["myproject"]);
    }

    #[test]
    fn test_detect_const_case() {
        let name = "MY_PROJECT";
        let (case_info, normalized_name) = CaseInfo::detect(name);
        assert_eq!(case_info.separator, Some('_'));
        assert_eq!(case_info.part_type, CaseType::UpperCase);
        assert_eq!(normalized_name.parts, vec!["my", "project"]);
    }

    #[test]
    fn test_convert_title_case() {
        let name = "my project";
        let (_, normalized_name) = CaseInfo::detect(name);
        let new_name = CaseInfo::detect("My Project").0.convert(normalized_name);
        assert_eq!(new_name, "My Project");
    }

    #[test]
    fn test_convert_upper_case() {
        let name = "my project";
        let (_, normalized_name) = CaseInfo::detect(name);
        let new_name = CaseInfo::detect("MY PROJECT").0.convert(normalized_name);
        assert_eq!(new_name, "MY PROJECT");
    }

    #[test]
    fn test_convert_lower_case() {
        let name = "My Project";
        let (_, normalized_name) = CaseInfo::detect(name);
        let new_name = CaseInfo::detect("my project").0.convert(normalized_name);
        assert_eq!(new_name, "my project");
    }

    #[test]
    fn test_convert_snake_case() {
        let name = "my project";
        let (_, normalized_name) = CaseInfo::detect(name);
        let new_name = CaseInfo::detect("my_project").0.convert(normalized_name);
        assert_eq!(new_name, "my_project");
    }

    #[test]
    fn test_convert_kebab_case() {
        let name = "my project";
        let (_, normalized_name) = CaseInfo::detect(name);
        let new_name = CaseInfo::detect("my-project").0.convert(normalized_name);
        assert_eq!(new_name, "my-project");
    }

    #[test]
    fn test_convert_mumble_case() {
        let name = "my project";
        let (_, normalized_name) = CaseInfo::detect(name);
        let new_name = CaseInfo::detect("myproject").0.convert(normalized_name);
        assert_eq!(new_name, "myproject");
    }

    #[test]
    fn test_convert_const_case() {
        let name = "my project";
        let (_, normalized_name) = CaseInfo::detect(name);
        let new_name = CaseInfo::detect("MY_PROJECT").0.convert(normalized_name);
        assert_eq!(new_name, "MY_PROJECT");
    }

    // Generate a test project structure with this layout
    // test-project
    // ├── test-dir-1
    // │   ├── test-dir-test-project
    // │   │   └── test-file-test-project.txt
    // │   └── test-file-2.txt
    // └── test-file-1.txt
    fn gen_test_project() {
        let test_dir = std::env::current_dir().unwrap().join("test-project");
        std::fs::create_dir_all(test_dir.join("test-dir-1/test-dir-test-project")).unwrap();
        std::fs::write(test_dir.join("test-dir-1/test-file-2.txt"), "Test Project").unwrap();
        std::fs::write(test_dir.join("test-file-1.txt"), "test-project").unwrap();
        std::fs::write(test_dir.join("test-dir-1/test-dir-test-project/test-file-test-project.txt"), "test_project").unwrap();
    }

    // Check if there is a project structure with this layout
    // copied-project
    // ├── test-dir-1
    // │   ├── test-dir-copied-project
    // │   │   └── test-file-copied-project.txt
    // │   └── copied-file-2.txt
    // └── copied-file-1.txt
    fn check_test_project() {
        let test_dir = std::env::current_dir().unwrap().join("copied-project");
        assert!(test_dir.exists());
        assert!(test_dir.join("test-dir-1").exists());
        assert!(test_dir.join("test-file-1.txt").exists());
        // Check the contents of test-file-1.txt
        let content = std::fs::read_to_string(test_dir.join("test-file-1.txt")).unwrap();
        assert_eq!(content, "copied-project");
        assert!(test_dir.join("test-dir-1/test-dir-copied-project").exists());
        assert!(test_dir.join("test-dir-1/test-file-2.txt").exists());
        let content = std::fs::read_to_string(test_dir.join("test-dir-1/test-file-2.txt")).unwrap();
        assert_eq!(content, "Copied Project");
        assert!(test_dir.join("test-dir-1/test-dir-copied-project/test-file-copied-project.txt").exists());
        let content = std::fs::read_to_string(test_dir.join("test-dir-1/test-dir-copied-project/test-file-copied-project.txt")).unwrap();
        assert_eq!(content, "copied_project");
    }

    #[test]
    fn test_complete() {
        gen_test_project();
        start(Args {
            name: "copied-project".to_string(),
            input: std::env::current_dir().unwrap().join("test-project")
        });
        check_test_project();
        std::fs::remove_dir_all(std::env::current_dir().unwrap().join("test-project")).unwrap();
        std::fs::remove_dir_all(std::env::current_dir().unwrap().join("copied-project")).unwrap();
    }
}