use std::io::{stdout, Write};
use time_tracker::{show_timer, Task, Timer};

/// A simple time tracking application for the command line interface.
///
/// This function is the entry point of the program.
/// It asks the user for a task name and then starts a timer.
/// When the user enters "stop", the timer is stopped and the time is printed.
/// The user can then enter another task name and the process repeats.
/// The user can enter "exit" to exit the program.
/// Upon exiting, the program prints the total time tracked for each task.
fn main() {
    println!("Welcome to the time tracker!");
    let prompt = "Enter a task name to start tracking it. Exit the program by typing 'exit'.\n";
    let mut ended = true;
    let mut tasks: Vec<Task> = Vec::new();
    let mut tasks_completed: Vec<String> = Vec::new();
    loop {
        if ended {
            print!("{}", prompt);
            print!("> ");
            stdout().flush().unwrap();

            let mut task_name = String::new();
            std::io::stdin().read_line(&mut task_name).unwrap();

            task_name = task_name.trim().to_string();
            if task_name == "exit" {
                break;
            }
            tasks.push(Task::new(&task_name));
            let task = tasks.last().unwrap();
            ended = false;
            println!("Started task '{}', stop the task with 'stop'", task.name);
            // show the timer until the user presses enter
            let mut new_timer = Timer::new();
            show_timer(&task.name, &mut new_timer);
        } else {
            let mut task = tasks.pop().unwrap();
            task.stop();
            ended = true;
            tasks_completed.push(format!("{}: {}", task.name, task));
            println!(
                "Task '{}' completed in {}.",
                task.name,
                task.time_tracked_string()
            );
        }
    }

    // output the tasks that were completed
    println!(
        "

Tasks completed:"
    );
    for task in tasks_completed {
        println!("{}", task);
    }

    println!(
        "

Goodbye!"
    );
}

// add tests for main()
#[cfg(test)]
mod tests {
    use time_tracker::ChildProcess;

    #[test]
    fn test_shows_welcome_message() {
        let mut child = ChildProcess::new();

        // send the exit command to the program
        child.write_to_stdin("exit");

        // check the output
        let mut output = String::new();
        child.read_to_string(&mut output);
        assert!(output.contains("Welcome to the time tracker!"));

        // make sure the program exited
        assert!(child.ended(), "Program exited with an error");
    }

    #[test]
    fn test_shows_goodbye_message() {
        let mut child = ChildProcess::new();

        // send the exit command to the program
        child.write_to_stdin("exit");

        // check the output
        let mut output = String::new();
        child.read_to_string(&mut output);
        assert!(output.contains("Goodbye!"));

        // make sure the program exited
        assert!(child.ended(), "Program exited with an error");
    }

    #[test]
    fn test_shows_prompt() {
        let mut child = ChildProcess::new();

        // send the exit command to the program
        child.write_to_stdin("exit");

        // check the output
        let mut output = String::new();
        child.read_to_string(&mut output);
        assert!(output.contains(
            "Enter a task name to start tracking it. Exit the program by typing 'exit'."
        ));

        // make sure the program exited
        assert!(child.ended(), "Program exited with an error");
    }

    #[test]
    fn test_shows_task_completed() {
        let mut child = ChildProcess::new();

        // send the commands to the program
        child.write_to_stdin("test task");
        child.write_to_stdin("exit");

        // check the output
        let mut output = String::new();
        child.read_to_string(&mut output);
        assert!(output.contains("Task 'test task' completed in"));

        // make sure the program exited
        assert!(child.ended(), "Program exited with an error");
    }

    #[test]
    fn test_shows_prompt_after_task() {
        let mut child = ChildProcess::new();

        // send the commands to the program
        child.write_to_stdin("test task");
        child.write_to_stdin("exit");

        // check the output
        let mut output = String::new();
        child.read_to_string(&mut output);
        let prompt = "Enter a task name to start tracking it. Exit the program by typing 'exit'.";

        // make sure there are two prompts
        let mut count = 0;
        for line in output.lines() {
            if line == prompt {
                count += 1;
            }
        }
        assert_eq!(count, 2);

        // make sure the program exited
        assert!(child.ended(), "Program exited with an error");

        // make sure the task was completed

        // make sure the program exited
        assert!(child.ended(), "Program exited with an error");
    }

    #[test]
    fn test_shows_tasks_completed() {
        let mut child = ChildProcess::new();

        // send the commands to the program
        child.write_to_stdin("test task");
        child.write_to_stdin("exit");

        // check the output
        let mut output = String::new();
        child.read_to_string(&mut output);
        assert!(output.contains("Tasks completed:"));
        assert!(output.contains("test task: 00:00:01"));

        // last line should be the goodbye message
        let lines: Vec<&str> = output.lines().collect();
        let mut goodbye_line = 0;
        for (i, line) in lines.iter().enumerate() {
            if line == &"Goodbye!" {
                goodbye_line = i;
                break;
            }
        }
        assert_eq!(goodbye_line, lines.len() - 1, "Actual output: {}", output);

        // make sure the program exited
        assert!(child.ended(), "Program exited with an error");
    }
}
