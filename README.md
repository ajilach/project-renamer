Project Renamer
===============

This program renames a project directory and all occurrences of the project name in the files
and directories. It also renames the files and directories to match the new project name.

Example
-------

Given a project with this structure:

```
test-project
├── test-dir-1
│   ├── test-dir-test-project
│   │   └── test-file-test-project.txt "test_project"
│   └── test-file-2.txt "Test Project"
└── test-file-1.txt "test-project"
```

Download the application binary from the GitHub Releases. You may need to make the binary executable first.

```
sudo chmod 777 project-renamer
```

Then Run the Project Renamer with this input:

```
./project-renamer --input "/path/to/test-project/" --name "copied-project"
```

You will receive a new directory adjacent to the original project with this structure:

```
copied-project
├── test-dir-1
│   ├── test-dir-copied-project
│   │   └── test-file-copied-project.txt "copied_project"
│   └── copied-file-2.txt "Copied Project"
└── copied-file-1.txt "copied-project"
```
