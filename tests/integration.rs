use assert_cmd::prelude::*;
use lazy_static::lazy_static;
use serial_test::serial;
use std::io::{Read, Write};
use std::process::Command;

lazy_static! {
    static ref TEMP_FILENAME: &'static str = "output_{}.log";
    static ref TEMP_DIR: std::path::PathBuf = std::env::temp_dir();
}

/// Struct representing a Child process, which can be use for testing.
struct TestChild {
    process: std::process::Child,
    file_path: std::path::PathBuf,
}

impl TestChild {
    /// Creates a new `TestChild` from a `std::process::Child`.
    fn new(name: String) -> TestChild {
        let filename = TEMP_FILENAME.replace("{}", &name);
        let file_path = TEMP_DIR.join(&filename);
        let output_file = std::fs::File::create(file_path).unwrap();
        let stdout = std::process::Stdio::from(output_file);
        let process = Command::cargo_bin("time-tracker")
            .unwrap()
            .stdin(std::process::Stdio::piped())
            .stdout(stdout)
            .env("TT_ENV", "test")
            .spawn()
            .unwrap();

        return TestChild {
            process,
            file_path: TEMP_DIR.join(&filename),
        };
    }

    /// Write a string to the stdin of the process.
    fn write(&mut self, input: &str, sleep_ms: u64) -> Result<(), Box<dyn std::error::Error>> {
        // check if input ends with a newline
        let mut input = input.to_string();
        if !input.ends_with('\n') {
            input.push('\n');
        }

        // write the input to the program
        self.process
            .stdin
            .as_mut()
            .unwrap()
            .write_all(input.as_bytes())?;

        self.sleep(sleep_ms)?;
        return Ok(());
    }

    /// Read the output file and return the contents.
    fn read(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let mut output = String::new();

        // read the contents of the output file in the tmp directory
        let mut output_file = std::fs::File::open(&self.file_path)?;
        output_file.read_to_string(&mut output)?;

        // close the file
        output_file.sync_all()?;

        return Ok(output);
    }

    /// Sleep for a given amount of milliseconds.
    fn sleep(&mut self, ms: u64) -> Result<(), Box<dyn std::error::Error>> {
        std::thread::sleep(std::time::Duration::from_millis(ms));
        return Ok(());
    }

    /// Remove all files in the tmp directory.
    #[allow(unreachable_code)]
    fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::remove_file(&self.file_path)?;

        return Ok(());
    }

    /// Kill the process.
    fn kill(&mut self) -> Result<(), std::io::Error> {
        return self.process.kill();
    }

    /// Kill all processes with the name `time-tracker`.
    fn kill_all(&mut self) -> Result<(), std::io::Error> {
        let mut child = Command::new("pkill")
            .arg("-f")
            .arg("time-tracker")
            .spawn()
            .expect("failed to execute process");

        let ecode = child.wait().expect("failed to wait on child");

        if !ecode.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to kill all processes.",
            ));
        }

        return Ok(());
    }
}

/// Drop the `TestChild` and kill the process.
impl Drop for TestChild {
    fn drop(&mut self) {
        println!("\nDropping TestChild...");

        // kill the process, or kill all processes if it fails
        match self.kill() {
            Ok(_) => println!("Killed process {}.", self.process.id()),
            Err(_) => self.kill_all().unwrap_or_else(|e| println!("Error: {}", e)),
        }

        // print the output file location
        println!("Output file: {}", self.file_path.display());

        println!("Dropped TestChild.");
    }
}

/// Creates a new [`TestChild`](TestChild) with the current function name passed.
macro_rules! TestChild {
    () => {
        TestChild::new(fn_name!())
    };
}

#[test]
#[serial]
fn test_child_process() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = TestChild!();
    assert!(child.process.id() > 0);
    assert!(child.process.try_wait()?.is_none());
    assert!(child.process.stdin.is_some());
    assert!(child.process.stdout.is_none());
    assert!(child.process.stderr.is_none());
    assert!(child.write("test", 100).is_ok());
    assert!(child.read().is_ok());
    let process_time = std::time::Instant::now();
    assert!(child.sleep(1000).is_ok());
    assert!(process_time.elapsed().as_millis() >= 1000);
    assert!(child.cleanup().is_ok());
    assert!(child.kill().is_ok());
    assert!(child.sleep(100).is_ok());
    Ok(())
}

#[test]
#[serial]
fn test_shows_tasks_completed() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = TestChild!();

    // send the commands to the program
    assert!(child.write("test task", 800).is_ok());
    assert!(child.write("stop", 500).is_ok());
    assert!(child.write("exit", 500).is_ok());

    // make sure the program exited
    child.kill()?;

    // read the contents of the file
    let output = child.read();

    assert!(output.is_ok());

    // check the output
    assert!(output?.contains("Tasks completed:\ntest task: 00:00:01"));

    // cleanup the tmp directory
    assert!(child.cleanup().is_ok());

    assert!(child.sleep(100).is_ok());
    return Ok(());
}

#[test]
#[serial]
fn test_shows_welcome_message() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = TestChild!();

    // wait for the welcome message
    assert!(child.sleep(500).is_ok());

    // kill the process
    assert!(child.kill().is_ok());

    // check the output
    let output = child.read();
    assert!(output.is_ok());
    assert!(output?.contains("Welcome to the time tracker!"));

    // cleanup the tmp directory
    assert!(child.cleanup().is_ok());

    return Ok(());
}

#[test]
fn test_shows_goodbye_message() {
    let mut child = TestChild!();

    // send the commands to the program
    assert!(child.write("exit", 500).is_ok());

    // kill the process
    assert!(child.kill().is_ok());

    // check the output
    let output = child.read();
    assert!(output.is_ok());
    assert!(output.unwrap().contains("Goodbye!"));

    // cleanup the tmp directory
    assert!(child.cleanup().is_ok());
}

// #[test]
// fn test_shows_prompt() {
//     return;
//     let mut child = ChildProcess::new();

//     // send the exit command to the program
//     child.write_to_stdin("exit");

//     // check the output
//     let mut output = String::new();
//     child.read_to_string(&mut output);
//     assert!(output
//         .contains("Enter a task name to start tracking it. Exit the program by typing 'exit'."));

//     // make sure the program exited
//     assert!(child.ended(), "Program exited with an error",);
// }

// #[allow(unreachable_code)]
// #[test]
// fn test_shows_task_completed() {
//     return;
//     let mut child = ChildProcess::new();

//     // send the commands to the program
//     child.write_to_stdin("test task");
//     child.write_to_stdin("exit");

//     // check the output
//     let mut output = String::new();
//     child.read_to_string(&mut output);
//     assert!(output.contains("Task 'test task' completed in"));

//     // make sure the program exited
//     child.kill();
//     assert!(child.ended(), "Program exited with an error");
// }

// #[allow(unreachable_code)]
// #[test]
// fn test_shows_prompt_after_task() {
//     return;
//     let mut child = ChildProcess::new();

//     // send the commands to the program
//     child.write_to_stdin("test task");
//     child.write_to_stdin("exit");

//     // check the output
//     let mut output = String::new();
//     child.read_to_string(&mut output);
//     let prompt = "Enter a task name to start tracking it. Exit the program by typing 'exit'.";

//     // make sure there are two prompts
//     let mut count = 0;
//     for line in output.lines() {
//         if line == prompt {
//             count += 1;
//         }
//     }
//     assert_eq!(count, 2);

//     // make sure the program exited
//     child.kill();
// }

// #[test]
// fn test_env_vars() {
//     // get the current environment variables
//     let env_vars = std::env::vars().collect::<std::collections::HashMap<String, String>>();

//     // show the environment variables, formatted as [KEY] = "VALUE"
//     assert!(
//         false,
//         "{:?}",
//         env_vars
//             .iter()
//             .map(|(k, v)| format!("[{}] = \"{}\"", k, v))
//             .collect::<Vec<String>>()
//             .join(
//                 "
//             "
//             )
//     );
// }

#[macro_export]
macro_rules! fn_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let mut name = type_name_of(f);
        name = &name[..name.len() - 3];

        // remove {{closure}} from the name
        let mut name = name.to_string().replace("::{{closure}}", "");

        // remove the module path
        name = name.split("::").last().unwrap_or(&name).to_string();
        name
    }};
}

#[cfg(test)]
mod test_macros {
    #[test]
    fn test_fn_name() {
        assert_eq!(fn_name!(), "test_fn_name");
    }
}
