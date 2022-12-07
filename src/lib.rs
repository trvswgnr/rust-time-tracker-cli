/*!
Library for advanced time tracking
Features:
- [ ] Display a menu bar with pages:
  - [ ] Time:
    - [ ] Show the current week as a row of days with the total time for each day. Highlight the selected day.
    - [ ] Default to the current day, showing the day's time entries:
      - [ ] Show a list of time entries for the day.
      - [ ] Allow the user to create a new time entry.
      - [ ] Allow the user to edit a time entry.
      - [ ] Allow the user to delete a time entry.
    - [ ] Allow the user to select a different day.
    - [ ] Allow the user to select a different week.
    - [ ] When creating or editing a time entry:
      - [ ] Allow the user to select a project.
      - [ ] Allow the user to select a task from the selected project.
      - [ ] Allow the user to enter a description.
      - [ ] Allow the user to start a timer.
      - [ ] Allow the user to stop a timer.
      - [ ] Allow the user to enter a duration.
  - [ ] Projects:
    - [ ] Display a list of projects.
    - [ ] Allow the user to create a new project.
    - [ ] Allow the user to edit a project.
    - [ ] Allow the user to delete a project.
    - [ ] Allow the user to select a project, entering a project detail page:
      - [ ] Show a list of tasks for the project.
      - [ ] Allow the user to create a new task.
      - [ ] Allow the user to edit a task.
      - [ ] Allow the user to delete a task.
- [ ] Start the program on the Time page, showing the current day.
- [ ] Store and retrieve data from an SQLite database.

Here are the libraries that are needed by this project:
- [ ] [clap](https://crates.io/crates/clap) - to parse command line arguments
- [ ] [chrono](https://crates.io/crates/chrono) - to work with dates and times
- [ ] [crossterm](https://crates.io/crates/crossterm) - to work with the terminal
- [ ] [r2d2](https://crates.io/crates/r2d2) - to manage a connection pool
- [ ] [r2d2_sqlite](https://crates.io/crates/r2d2_sqlite) - to manage a connection pool for SQLite
- [ ] [rusqlite](https://crates.io/crates/rusqlite) - to work with SQLite
- [ ] [serde](https://crates.io/crates/serde) - to serialize and deserialize data
- [ ] [serde_json](https://crates.io/crates/serde_json) - to serialize and deserialize data to and from JSON
- [ ] [serde_yaml](https://crates.io/crates/serde_yaml) - to serialize and deserialize data to and from YAML
*/

#![allow(unused_imports, deprecated)]
use chrono::{
    self, prelude::*, Date, DateTime, Datelike, Days, Duration as ChronoDuration, DurationRound,
    FixedOffset, IsoWeek, Local, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, NaiveWeek,
    Timelike, Utc, Weekday,
};
// import all available from the clap crate (name them)
use clap::{
    Arg, ArgAction, ArgGroup, ArgMatches, Args, ColorChoice, Command, CommandFactory,
    Error as ClapError, FromArgMatches, Id, Parser, Subcommand, ValueEnum, ValueHint,
};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    execute, queue,
    style::{self, Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, types::ToSql, Connection, NO_PARAMS};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    collections::{
        hash_map::Entry, BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque,
    },
    error::Error,
    io::{self, Write},
    process,
    sync::{
        atomic::{
            AtomicBool, AtomicUsize,
            Ordering::{self, SeqCst},
        },
        mpsc::{self, channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

// Let's start by creating our tests.
// The first thing we need to do is create a module for our tests.
// This way we can build our features to pass the tests.

// Let's create a test to check our program's flow.
#[cfg(test)]
mod tests {
    // We'll need to import the necessary libraries.
    use super::*;
    use chrono::{Datelike, Timelike};
    use std::{
        io::stdout,
        time::{SystemTime, UNIX_EPOCH},
    };

    // Tests:
    // he program starts on the Time page.
    // The main menu is displayed.
    // The current week is displayed as a row of days with the total time for each day. The current day is highlighted in the row of days. The week's total time is displayed at the end of the row of days. The dates are above each day.
    // We'll also check that the current day's time entries are displayed in a list.
    // We'll also check that the current day's total time is displayed at the bottom of the list.
    // The sub-menu is displayed at the bottom of the screen.
    /*
    Here's how the start of the program should look:

    Welcome to the Time Tracker!
    ------------------------------------------------------------
    Time (t) | Projects (p) | Quit (q) | Help (h)
    ------------------------------------------------------------

    | 12/01 | 12/02 | 12/03 | 12/04 | 12/05 | 12/06 | 12/07 |  00:00  |
    |  Mon  |  Tue  |  Wed  |  Thu  |  Fri  |  Sat  |  Sun  |  Total  |
    | 00:00 | 00:00 | 00:00 | 01:30 | 00:00 | 00:00 | 00:00 |  00:00  |

    Thursday, December 4, 2020:
    1. 01:00 - Test Entry 1 (Test Project 1: Test Task 1)
    2. 00:30 - Test Entry 2 (Test Project 1: Test Task 2)

    ------------------------------------------------------------
    New (n) | Edit (e) | Delete (d) | Change Date (c)
    */
    #[test]
    fn test_start_time_page_display() {
        // First let's create a new thread to run our program.
        // We'll need to create a channel to communicate with the thread.
        let (tx, rx) = channel();
        // We'll need to create a thread to run our program.
        let handle = thread::spawn(move || {});
        // First let's mock the database.
        // We'll need to create a connection to the database.
        let conn = Connection::open_in_memory().unwrap();
        // We'll need to create the tables.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT
            )",
            NO_PARAMS,
        )
        .unwrap();
        // We'll need to create the tasks table.
        // It should have a unique id.
        // It should have a project_id.
        // It should have a name.
        // It should have a description.
        // It should have a field that contain the ids of the time entries (we'll use a string to store the ids and separate them with commas).
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                project_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                time_entries TEXT,
                FOREIGN KEY(project_id) REFERENCES projects(id)
            )",
            NO_PARAMS,
        )
        .unwrap();

        // the created_at field should be default to the current time (as a unix timestamp).
        conn.execute(
            "CREATE TABLE IF NOT EXISTS time_entries (
                id INTEGER PRIMARY KEY,
                description TEXT NOT NULL,
                task_id INTEGER NOT NULL,
                duration INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY(task_id) REFERENCES tasks(id)
            )",
            NO_PARAMS,
        )
        .unwrap();
        // We'll need to insert some data into the database.
        // We'll need to insert some projects.
        conn.execute(
            "INSERT INTO projects (name, description) VALUES (?1, ?2)",
            params!["Test Project 1", "This is a test project."],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO projects (name, description) VALUES (?1, ?2)",
            params!["Test Project 2", "This is a test project."],
        )
        .unwrap();
        // We'll need to insert some tasks.
        conn.execute(
            "INSERT INTO tasks (project_id, name, description) VALUES (?1, ?2, ?3)",
            params![1, "Test Task 1", "This is a test task."],
        );
        conn.execute(
            "INSERT INTO tasks (project_id, name, description) VALUES (?1, ?2, ?3)",
            params![1, "Test Task 2", "This is a test task."],
        );
        // We'll need to insert some time entries.
        conn.execute(
            "INSERT INTO time_entries (description, task_id, duration, created_at) VALUES (?1, ?2, ?3, ?4)",
            params!["Test Entry 1", 1, 3600, "2020-12-04 01:00:00"],
        );
        conn.execute(
            "INSERT INTO time_entries (description, task_id, duration, created_at) VALUES (?1, ?2, ?3, ?4)",
            params!["Test Entry 2", 2, 1800, "2020-12-04 01:30:00"],
        );

        // We'll need to create a new instance of the App struct.
        let mut time_tracker = App::new(conn);

        // Then we'll need to call the start method.
        time_tracker.run();

        // We'll need to check that the program starts on the Time page.
        assert_eq!(time_tracker.current_page, Page::Time);

        // We'll need to check that the main menu is displayed.
        assert_eq!(
            time_tracker.main_menu,
            vec!["Time (t)", "Projects (p)", "Quit (q)", "Help (h)"]
        );
        // Make sure the main menu contents are in the stdout.
        let mut output = String::new();
        // We'll need to get the stdout.
        let stdout = io::stdout();
        // We'll need to get the lock.
        let mut handle = stdout.lock();
        // Check that the main menu is in the stdout.
        handle.read_to_string(&mut output).unwrap();

        // We'll need to check that the current week is displayed as a row of days with the total time for each day.
        // We'll also check that the current day is highlighted in the row of days.
        // We'll also check that the week's total time is displayed at the end of the row of days.
        // We'll also check that the dates are above each day.
        // We'll need to get the current date.
        let current_date = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Let's check
    }
}
