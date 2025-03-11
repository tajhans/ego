# ego

Ego is a simple productivity tracking tool that monitors your coding sessions by tracking the time spent and lines of code written in a project directory.

## Installation

### Prerequisites
- Rust and Cargo (install from [rustup.rs](https://rustup.rs))

### Using Cargo
- Install using the GitHub repository:
  ```
  cargo install --git https://github.com/tajhans/ego
  ```

### Building from Source
1. Clone the repository:
   ```
   git clone https://github.com/tajhans/ego
   cd ego
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. The executable will be available at `target/release/ego`

4. Optionally, add it to your PATH:
   ```
   cp target/release/ego ~/.local/bin/  # or another directory in your PATH
   ```

## Usage

Ego provides two main commands:

### Start a Session
```
ego start <PROJECT_DIRECTORY>
```
This will start tracking your coding session in the specified directory.

### End a Session
```
ego end
```
This will end the current session and display statistics about your coding session, including:
- Session duration
- Initial line count
- Final line count
- Lines written (added or removed)

## Features

- Tracks time spent on a project
- Counts lines of code in various file types (including .rs, .py, .js, .html, .css, and many more)
- Provides a clean terminal UI for session statistics
- Simple and minimalistic interface

## License

[MIT](LICENSE)
