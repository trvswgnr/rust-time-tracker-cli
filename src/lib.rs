use lazy_static::lazy_static;
use rpassword::read_password;
use std::{
    io,
    io::{Read, Write},
    thread,
    time::Instant,
};

lazy_static! {
    static ref DAYS_DIVISOR: u64 = {
        let days_divisor = 60 * 60 * 24; // equivalent to 86400
        days_divisor
    };

    static ref HOURS_DIVISOR: u64 = {
        let hours_divisor = 60 * 60; // equivalent to 3600
        hours_divisor
    };

    static ref MINUTES_DIVISOR: u64 = {
        let minutes_divisor = 60; // equivalent to 60
        minutes_divisor
    };

    static ref DAY_IN_SECONDS: u64 = {
        let day_in_seconds = 86400;
        day_in_seconds
    };

    static ref HOUR_IN_SECONDS: u64 = {
        let hour_in_seconds = 3600;
        hour_in_seconds
    };

    static ref MINUTE_IN_SECONDS: u64 = {
        let minute_in_seconds = 60;
        minute_in_seconds
    };
}

fn get_clock_format(elapsed: u64) -> String {
    let hours = elapsed / *HOURS_DIVISOR;
    let minutes = (elapsed % *HOURS_DIVISOR) / *MINUTES_DIVISOR;
    let seconds = elapsed % *MINUTES_DIVISOR;
    return vec![hours, minutes, seconds]
        .iter()
        .map(|time_unit| format!("{:02}", time_unit))
        .collect::<Vec<String>>()
        .join(":");
}

/// A timer that can be used to track the time elapsed since it was started.
///
/// # Examples
///
/// ```no_run
/// let mut timer = time_tracker::Timer::new();
/// std::thread::sleep(std::time::Duration::from_secs(5));
/// timer.update();
///
/// let elapsed = timer.elapsed();
/// println!("{} seconds", elapsed); // -> "5 seconds"
/// ```
#[derive(Clone, Copy)]
pub struct Timer {
    /// When the timer was started.
    start: Instant,
    /// When the timer was stopped.
    end: Instant,
}

impl Timer {
    /// Creates a new `Timer` and starts it.
    pub fn new() -> Timer {
        Timer {
            start: Instant::now(),
            end: Instant::now(),
        }
    }

    /// Updates the timer by setting the `end` field to the current time.
    pub fn update(&mut self) {
        self.end = Instant::now();
    }

    /// Gets the time elapsed since the timer was started (in seconds).
    pub fn elapsed(&self) -> u64 {
        return self.end.duration_since(self.start).as_secs();
    }
}

/// Formats trait to display the time elapsed in a clock format.
impl std::fmt::Display for Timer {
    /// Formats the timer as 'HH:MM:SS'.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let elapsed = self.elapsed();
        let clock_format = get_clock_format(elapsed);
        write!(f, "{}", clock_format)
    }
}

/// A single task that time is tracked for.
///
/// # Examples
/// ```no_run
/// let name = String::from("Task 1");
/// let mut task = time_tracker::Task::new(&name);
/// let task_name = &task.name;
/// task.stop();
/// let seconds = task.time_tracked_seconds();
/// let time_tracked = task.time_tracked_string();
/// ```
pub struct Task {
    /// The name of the task.
    pub name: String,
    start: Instant,
    end: Instant,
}

impl Task {
    /// Creates a new task with the given name.
    ///
    /// # Examples
    /// ```no_run
    /// let name = String::from("Task 1");
    /// let task = time_tracker::Task::new(&name);
    /// ```
    pub fn new(name: &String) -> Task {
        Task {
            name: name.to_string(),
            start: Instant::now(),
            end: Instant::now(),
        }
    }

    /// Stops the task by setting the end time to the current time.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let name = String::from("Task 1");
    /// let mut task = time_tracker::Task::new(&name);
    /// std::thread::sleep(std::time::Duration::from_secs(5));
    /// task.stop();
    /// ```
    pub fn stop(&mut self) {
        self.end = Instant::now();
    }

    /// Gets the total time tracked since the task was started (in seconds).
    ///
    /// If the task is still running, the elapsed time will be the time elapsed since the task was started until the current time.
    /// If the task has been stopped, the elapsed time will be the time elapsed since the task was started until the task was stopped.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let name = String::from("Task 1");
    /// let mut task = time_tracker::Task::new(&name);
    /// std::thread::sleep(std::time::Duration::from_secs(1));
    /// task.stop();
    /// let time_tracked = task.time_tracked_seconds();
    /// println!("Time tracked: {} seconds", time_tracked); // -> Time elapsed: 1 seconds
    /// ```
    pub fn time_tracked_seconds(&self) -> u64 {
        return self.end.duration_since(self.start).as_secs();
    }

    /// Gets  the amount of time tracked as X Days, X Hours, Y Minutes, and Z Seconds.
    ///
    /// If a time unit is 0, it will not be included in the string.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let name = String::from("Task 1");
    /// let mut task = time_tracker::Task::new(&name);
    /// std::thread::sleep(std::time::Duration::from_secs(1));
    /// task.stop();
    /// let duration = task.time_tracked_string();
    /// println!("{}", duration); // -> 0 Days, 0 Hours, 0 Minutes, 1 Second
    /// ```
    pub fn time_tracked_string(&self) -> String {
        // get the total number of seconds.
        let total_seconds = self.time_tracked_seconds();

        // get the number of days
        let days = total_seconds / *DAYS_DIVISOR;

        // get the number of hours left over.
        let hours = (total_seconds % *DAYS_DIVISOR) / *HOURS_DIVISOR;

        // get the number of minutes left over.
        let minutes = (total_seconds % *HOURS_DIVISOR) / *MINUTES_DIVISOR;

        // get the number of seconds left over.
        let seconds = total_seconds % *MINUTES_DIVISOR;

        // create a vector to hold the time units.
        let mut time_units: Vec<String> = Vec::new();

        // add the days to the vector.
        if days > 0 {
            time_units.push(format!("{} Day{}", days, if days > 1 { "s" } else { "" }));
        }

        // add the hours to the vector.
        if hours > 0 {
            time_units.push(format!(
                "{} Hour{}",
                hours,
                if hours > 1 { "s" } else { "" }
            ));
        }

        // add the minutes to the vector.
        if minutes > 0 {
            time_units.push(format!(
                "{} Minute{}",
                minutes,
                if minutes > 1 { "s" } else { "" }
            ));
        }

        // add the seconds to the vector.
        if seconds > 0 {
            time_units.push(format!(
                "{} Second{}",
                seconds,
                if seconds > 1 { "s" } else { "" }
            ));
        }

        // create a string to hold the output.
        let mut output = String::new();

        // loop through the time units.
        for (index, time_unit) in time_units.iter().enumerate() {
            // if this is the last time unit, we add "and" before it, unless there is only one time unit.
            if index == time_units.len() - 1 && time_units.len() > 1 {
                output.push_str("and ");
            }

            // add the time unit to the output.
            output.push_str(time_unit);

            // if this is not the last time unit, add a comma.
            if index != time_units.len() - 1 {
                output.push_str(", ");
            }
        }

        // if there are only two time units, remove the comma.
        if time_units.len() == 2 {
            output = output.replace(", ", " ");
        }

        return output;
    }
}

/// Format trait for displaying the time tracked in a clock format.
impl std::fmt::Display for Task {
    /// Formats the task as 'HH:MM:SS'.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let elapsed = self.time_tracked_seconds();
        let clock_format = get_clock_format(elapsed);
        write!(f, "{}", clock_format)
    }
}

/// Shows a timer for the given task name.
///
/// Displays a timer for the given task name as 'Task Name: 00:00:00'.
/// The timer will update every second until the user types 'stop'.
///
/// ! When testing, the rpassword::read_password() function will immediately return 'stop' because of error handling.
///
/// # Examples
///
/// ```no_run
/// let task_name = String::from("Task 1");
/// let mut timer = time_tracker::Timer::new();
/// time_tracker::show_timer(&task_name, &mut timer);
/// ```
pub fn show_timer(task_name: &String, timer: &mut Timer) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut invalid = false;
    // holds the input while the timer is running
    thread::spawn(move || {
        // read input from stdin silently so that the user doesn't see what they type
        // (prevents ugly output when the user types while the timer is running)
        let input = match read_password() {
            Ok(task_name) => task_name,
            Err(_) => String::from("stop"),
        };
        tx.send(input).unwrap();
    });
    // loop until the user has typed 'stop'
    loop {
        timer.update();
        // replace the timer and the user input with the new timer and user input
        // print the task name and the timer
        print!("\r{}: {}", task_name, timer);
        io::stdout().flush().unwrap();
        // check if notification is empty, if not, print it
        print!("\n\r{}", "> ");
        io::stdout().flush().unwrap();

        // wait for 1 second
        thread::sleep(std::time::Duration::from_secs(1));

        // remove the last line
        print!("\x1B[1A");

        if let Ok(input) = rx.try_recv() {
            if input.trim() == "stop" {
                break;
            } else {
                invalid = true;
                break;
            }
        }
    }

    if invalid {
        println!("Invalid input. Please type 'stop' to stop the timer.");
        show_timer(task_name, timer);
    }
}

/// A struct that represents a child process.
///
/// Contains the child process itself, as well as the
/// stdin and stdout of the child process.
///
/// # Examples
///
/// ```no_run
/// use time_tracker::ChildProcess;
///
/// let mut child_process = ChildProcess::new();
///
/// child_process.write_to_stdin("exit");
///
/// let mut buffer = String::new();
/// child_process.read_to_string(&mut buffer);
/// ```
pub struct ChildProcess {
    /// The child process.
    pub child: std::process::Child,
    /// The stdin of the child process.
    pub stdin: Option<std::process::ChildStdin>,
    /// The stdout of the child process.
    pub stdout: Option<std::process::ChildStdout>,
}

impl ChildProcess {
    /// Creates a new `ChildProcess`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut child_process = time_tracker::ChildProcess::new();
    /// ```
    pub fn new() -> Self {
        let child: std::process::Child = std::process::Command::new("cargo")
            .arg("run")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        return ChildProcess {
            child,
            stdin: None,
            stdout: None,
        };
    }

    /// Waits for the child process to exit and returns its exit status.
    /// If the child process has already exited, this function returns immediately.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut child_process = time_tracker::ChildProcess::new();
    /// let exit_status = child_process.wait();
    /// ```
    pub fn wait(&mut self) -> Result<std::process::ExitStatus, std::io::Error> {
        return self.child.wait();
    }

    /// Writes the given string to the child process's stdin.
    ///
    /// @note: appends a newline to the string to simulate the user pressing enter.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut child_process = time_tracker::ChildProcess::new();
    /// child_process.write_to_stdin("exit");
    /// ```
    pub fn write_to_stdin(&mut self, input: &str) {
        let mut input = String::from(input);
        if !input.ends_with('\n') {
            input.push('\n');
        }
        // convert &str to &[u8]
        let input = input.as_bytes();
        self.child.stdin.as_mut().unwrap().write_all(input).unwrap();
    }

    /// Reads from the child process's stdout and writes it to the given String buffer.
    ///
    /// # Arguments
    /// * `target` - A mutable reference to a [`String`](std::string::String) buffer to write the child process's stdout to.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut child_process = time_tracker::ChildProcess::new();
    /// let mut buffer = String::new();
    /// child_process.read_to_string(&mut buffer);
    /// ```
    pub fn read_to_string(&mut self, target: &mut String) {
        self.child
            .stdout
            .as_mut()
            .unwrap()
            .read_to_string(target)
            .unwrap();
    }

    /// Check if the child process was terminated successfully.
    ///
    /// @returns [`bool`](std::primitive::bool) - `true` if the child process was terminated successfully, `false` otherwise.
    ///
    /// [`true`](std::primitive::bool) if the child process has ended, [`false`](std::primitive::bool) otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut child_process = time_tracker::ChildProcess::new();
    /// let ended = child_process.ended();
    /// if ended {
    ///    println!("The child process has ended.");
    /// } else {
    ///   println!("The child process is still running.");
    /// }
    /// ```
    pub fn ended(&mut self) -> bool {
        return self.wait().unwrap().success();
    }
}

#[cfg(test)]
mod tests_task {
    use super::*;

    #[test]
    fn creates_new_task() {
        let mut task = Task::new(&"Test".to_string());
        assert_eq!(task.name, "Test");
        task.start = Instant::now() - std::time::Duration::from_secs(1);
        task.stop();
        assert_eq!(task.time_tracked_seconds(), 1);
    }

    #[test]
    fn correct_duration_as_string() {
        let task_name = "Test".to_string();
        let mut task = Task::new(&task_name);
        task.start = Instant::now() - std::time::Duration::from_secs(1);
        task.stop();
        assert_eq!(task.time_tracked_string(), "1 Second");

        let mut task = Task::new(&task_name);
        task.start = Instant::now() - std::time::Duration::from_secs(61);
        task.stop();
        assert_eq!(task.time_tracked_string(), "1 Minute and 1 Second");

        // simulate 1 minute and 2 seconds.
        let mut task = Task::new(&task_name);
        task.start = Instant::now() - std::time::Duration::from_secs(62);
        task.stop();
        assert_eq!(task.time_tracked_string(), "1 Minute and 2 Seconds");

        // simulate 2 minutes.
        let mut task = Task::new(&task_name);
        task.start = Instant::now() - std::time::Duration::from_secs(120);
        task.stop();
        assert_eq!(task.time_tracked_string(), "2 Minutes");

        // simulate 1 day.
        let mut task = Task::new(&task_name);
        task.start = Instant::now() - std::time::Duration::from_secs(86400);
        task.stop();
        assert_eq!(task.time_tracked_string(), "1 Day");

        // simulate 1 day, 1 hour, 1 minute, and 39 seconds.
        let mut task = Task::new(&task_name);
        let days = *DAY_IN_SECONDS;
        let hours = *HOUR_IN_SECONDS;
        let minutes = *MINUTE_IN_SECONDS;
        let seconds = 39;
        task.start =
            Instant::now() - std::time::Duration::from_secs(days + hours + minutes + seconds);
        task.stop();
        assert_eq!(
            task.time_tracked_string(),
            "1 Day, 1 Hour, 1 Minute, and 39 Seconds"
        );

        // simulate 1 day, 1 hour, 1 minute, and 1 second.
        let mut task = Task::new(&task_name);
        let days = *DAY_IN_SECONDS;
        let hours = *HOUR_IN_SECONDS;
        let minutes = *MINUTE_IN_SECONDS;
        let seconds = 1;
        task.start =
            Instant::now() - std::time::Duration::from_secs(days + hours + minutes + seconds);
        task.stop();
        assert_eq!(
            task.time_tracked_string(),
            "1 Day, 1 Hour, 1 Minute, and 1 Second"
        );

        // simulate 4 hours, 45 minutes, and 53 seconds.
        let mut task = Task::new(&task_name);
        let hours = *HOUR_IN_SECONDS * 4;
        let minutes = *MINUTE_IN_SECONDS * 45;
        let seconds = 53;
        task.start = Instant::now() - std::time::Duration::from_secs(hours + minutes + seconds);
        task.stop();
        assert_eq!(
            task.time_tracked_string(),
            "4 Hours, 45 Minutes, and 53 Seconds"
        );

        // simulate 4 days and 8 hours.
        let mut task = Task::new(&task_name);
        let days = *DAY_IN_SECONDS * 4;
        let hours = *HOUR_IN_SECONDS * 8;
        task.start = Instant::now() - std::time::Duration::from_secs(days + hours);
        task.stop();
        assert_eq!(task.time_tracked_string(), "4 Days and 8 Hours");
    }
}

#[cfg(test)]
mod util {
    use super::*;
    #[test]
    fn test_get_clock_format() {
        assert_eq!(get_clock_format(0), "00:00:00");
        assert_eq!(get_clock_format(1), "00:00:01");
        assert_eq!(get_clock_format(60), "00:01:00");
        assert_eq!(get_clock_format(3600), "01:00:00");
        assert_eq!(get_clock_format(86400), "24:00:00");
        assert_eq!(get_clock_format(86401), "24:00:01");
        assert_eq!(get_clock_format(86460), "24:01:00");
        assert_eq!(get_clock_format(99999), "27:46:39");
    }
}
