# Uffice
[![uffice](https://github.com/usadson/uffice/actions/workflows/build.yaml/badge.svg)](https://github.com/usadson/uffice/actions/workflows/build.yaml)

Uffice is an alpha-stage word processor, working on compatibility with WordprocessingML format of the [Office Open Specification (ECMA-376)](https://www.ecma-international.org/publications-and-standards/standards/ecma-376/).

<p align="center">
   <img src="docs/screenshot/quick-overview.png" alt="Demo of the uffice application on Windows." width="641" height="380">
</p>

## Installation
As the software is alpha-stage, the user experience is yet to be. You can simply clone this repository and use [Cargo](https://doc.rust-lang.org/cargo/) to build the project.

```sh
# Clone using HTTP:
git clone https://github.com/usadson/uffice

# or using SSH:
git clone git@github.com:usadson/uffice.git

cd uffice

# Build the application
cargo build
```

## Usage
The application is currently in alpha-stage, and is not yet ready for production use. However, you can still use it to open documents. Editing is not yet supported, but work is being done to add this feature.

Currently, files can be opened by specifying their path as arguments to the application. For example, to open the `test.docx` file, you can run the following command:
```sh
cargo run -- test.docx
```

A user-interface is still being worked on, but you can drag & drop documents onto the application window to open them.

## UX Checklist
The following checklist documents the requirements that have to put in place to help aid new users, installations and usages of the application.

- [x] Add a CI/CD pipeline
- [ ] Provide production-ready binaries
- [ ] Publish a release
- [ ] Add a user interface to open files
- [ ] Add a greeting screen, welcoming the user to open a document
- [ ] Remove the `UFFICE_TEST_FILE` environment variable

## Reference Material
- [ECMA-376](https://www.ecma-international.org/publications-and-standards/standards/ecma-376/)

## Licensing
This project is currently licensed under a BSD-2-Clause-compatible [license](LICENSE.md), but uses various dependencies, with distinct licenses each.

## Legal
For any legal questions or issues, including patents and IP, please contact me via email or [Twitter](https://twitter.com/TAGerritsen).
