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
    assert!(output
        .contains("Enter a task name to start tracking it. Exit the program by typing 'exit'."));

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
