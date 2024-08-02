# Job File Parser

Job File Parser is a Rust-based tool designed for parsing both legacy binary job files and modern XML job files used by the Windows Task Scheduler. This tool provides comprehensive details about the job files, which is essential for forensic analysis and understanding task scheduling behavior in Windows environments.

## Features

- Parses binary job files with detailed information.
- Parses modern XML job files used by Windows Task Scheduler.
- Provides human-readable output of job details.
- Supports batch processing of job files in a directory.

## Dependencies

- Rust 1.56 or later
- `getopts`
- `encoding_rs`
- `encoding_rs_io`
- `quick_xml`
- `serde`

## Installation

1. Ensure you have Rust installed. If not, install it from [rust-lang.org](https://www.rust-lang.org/).
2. Clone the repository:

    ```sh
    git clone https://github.com/mehrn0ush/jobfileparser.git
    cd jobfileparser
    ```

3. Build the project:

    ```sh
    cargo build --release
    ```

## Usage

The tool supports two primary modes: parsing a single job file or parsing all job files in a directory.

### Command-Line Options

- `-h, --help`: Print this help menu.
- `-f, --file <FILE>`: Set job file to parse.
- `-d, --dir <DIR>`: Set directory of job files to parse.

### Examples

#### Parsing a Single Job File

To parse a single job file (either binary or XML):

```sh
./target/release/jobfileparser -f path/to/your/jobfile.job

