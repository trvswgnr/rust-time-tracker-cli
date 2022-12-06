//! This program is a simple time tracking application for the command line interface.
//!
//! It asks the user for a task name and then starts a timer.
//! When the user enters "stop", the timer is stopped and the time is printed.
//! The user can then enter another task name and the process repeats.
//! The user can enter "exit" to exit the program.
//! Upon exiting, the program prints the total time tracked for each task.

use std::io::{stdout, Write};
use timetracker::{Task, Timer};

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
            task.show_timer(&mut new_timer);
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
