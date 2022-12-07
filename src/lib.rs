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

/// Tests that the output is correct at various stages of the program.
#[cfg(test)]
mod test_displays_correct_output {
    use super::*;

    /**
    Tests that the program starts on the Time page and displays the correct output.
    - The program starts on the Time page.
    - The main menu is displayed.
    - The current week is displayed as a row of days with the total time for each day.
      - The current day is highlighted in the row of days.
      - The week's total time is displayed at the end of the row of days.
      - The dates are above each day.
    - The current day's time entries are displayed in a list.
    - The current day's total time is displayed at the bottom of the list.
    - The sub-menu is displayed at the bottom of the screen.

    Here's how the start of the program should look:
    ```sh
    Welcome to the Time Tracker!
    ---------------------------------------------
    Time (t) | Projects (p) | Quit (q) | Help (h)
    ---------------------------------------------

    | 12/01 | 12/02 | 12/03 | 12/04 | 12/05 | 12/06 | 12/07 |  00:00  |
    |  Mon  |  Tue  |  Wed  |  Thu  |  Fri  |  Sat  |  Sun  |  Total  |
    | 00:00 | 00:00 | 00:00 | 01:30 | 00:00 | 00:00 | 00:00 |  01:30  |

    Thursday, December 4, 2020:

    1. 01:00 - Test Entry 1 (Test Project 1: Test Task 1)
    2. 00:30 - Test Entry 2 (Test Project 1: Test Task 2)

    --------------------------------------------------------------------
    New (n) | Edit (e) | Delete (d) | Change Date (c) | Go to Today (g)
    ```
    */
    #[test]
    fn test_start_time_page_display() {
        // instead of using the real stdout, we'll use a mock.
        let mut stdout = io::Cursor::new(Vec::new());

        // here's how we'll write to the mock:
        write!(stdout, "Hello, world!").unwrap();

        // here's how we'll read from the mock:
        let output = String::from_utf8(stdout.into_inner()).unwrap();
        assert_eq!(output, "Hello, world!");

        // mock the database connection (the real database connection will use a file on the user's computer)
        let conn = Connection::open_in_memory().unwrap();

        // create the tables in the database:
        // - id is the unique id of the project.
        // - name is the name of the project.
        // - description is the description of the project.
        // - tasks is a list of the tasks that belong to the project, stored as a string of comma-separated ids.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    description TEXT,
                    tasks TEXT
                )",
            NO_PARAMS,
        )
        .unwrap();
        // id is the unique id of the task.
        // project_id is the id of the project that the task belongs to.
        // name is the name of the task.
        // description is the description of the task.
        // time_entries is a list of the time entries that belong to the task, stored as a string of comma-separated ids.
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

        // id is the unique id of the time entry.
        // description is the description of the time entry.
        // task_id is the id of the task that the time entry belongs to.
        // start is the start time of the time entry, stored as a unix timestamp.
        // end is the end time of the time entry, stored as a unix timestamp.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS time_entries (
                    id INTEGER PRIMARY KEY,
                    description TEXT NOT NULL,
                    task_id INTEGER NOT NULL,
                    start INTEGER NOT NULL,
                    end INTEGER,
                    FOREIGN KEY(task_id) REFERENCES tasks(id)
                )",
            NO_PARAMS,
        )
        .unwrap();
        // insert some data into the database.
        // insert some projects.
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
        // insert some tasks.
        conn.execute(
            "INSERT INTO tasks (project_id, name, description) VALUES (?1, ?2, ?3)",
            params![1, "Test Task 1", "This is a test task."],
        );
        conn.execute(
            "INSERT INTO tasks (project_id, name, description) VALUES (?1, ?2, ?3)",
            params![1, "Test Task 2", "This is a test task."],
        );
        // insert some time entries.
        conn.execute(
            "INSERT INTO time_entries (description, task_id, start, end) VALUES (?1, ?2, ?3, ?4)",
            params![
                "Test Entry 1", // description
                1,              // task_id
                1607116800,     // start - 12/04/2020 @ 12:00am (UTC)
                1607120400      // end - 12/04/2020 @ 1:00am (UTC)
            ],
        );
        conn.execute(
            "INSERT INTO time_entries (description, task_id, start, end) VALUES (?1, ?2, ?3, ?4)",
            params![
                "Test Entry 2", // description
                2,              // task_id
                1607120400,     // start - 12/04/2020 @ 1:00am (UTC)
                1607121000      // end - 12/04/2020 @ 1:30am (UTC)
            ],
        );

        // update the tasks in the database to include the time entries.
        conn.execute(
            "UPDATE tasks SET time_entries = ?1 WHERE id = ?2",
            params!["1", 1],
        );
        conn.execute(
            "UPDATE tasks SET time_entries = ?1 WHERE id = ?2",
            params!["2", 2],
        );

        // update the projects in the database to include the tasks.
        conn.execute(
            "UPDATE projects SET tasks = ?1 WHERE id = ?2",
            params!["1,2", 1],
        );

        // create a new instance of the App struct.
        let mut time_tracker = App::new(conn, stdout);

        // Then we'll need to call the start method.
        time_tracker.run();

        // check that the program starts on the Time page.
        assert_eq!(time_tracker.current_page, Page::Time);

        // check that the welcome message is displayed.
        let welcome_message = "Welcome to the Time Tracker!";
        assert!(time_tracker.stdout.contains(welcome_message));

        // check that the main menu is displayed.
        let main_menu = "Time (t) | Projects (p) | Quit (q) | Help (h)";
        assert!(time_tracker.stdout.contains(main_menu));

        // * we'll need to get the current date in our app, but we'll mock it here as a Thursday, December 4, 2020.
        let current_date = NaiveDate::from_ymd(2020, 12, 4);

        time_tracker.go_to_day(current_date); // show the time page for the specified date.

        // make sure the current date is displayed.
        assert!(time_tracker
            .stdout
            .contains(&format!("{}", current_date.format("%A, %B %e, %Y")))); // Thursday, December 4, 2020

        // make the week dates are displayed.
        let week_dates = "| 12/01 | 12/02 | 12/03 | 12/04 | 12/05 | 12/06 | 12/07 |";
        assert!(time_tracker.stdout.contains(week_dates));

        // make sure the week is displayed.
        let week = "|  Mon  |  Tue  |  Wed  |  Thu  |  Fri  |  Sat  |  Sun  |  Total  |";
        assert!(time_tracker.stdout.contains(week));

        // TODO: We'll need to check that the current day is highlighted somehow, maybe with a different color.

        // check that the times for each day are displayed below the dates, and the total time for the day is displayed at the end of the row.
        let day_times =
            "|  00:00  |  00:00  |  00:00  |  01:30  |  00:00  |  00:00  |  00:00  |  01:30  |";
        assert!(time_tracker.stdout.contains(day_times));

        // check that the total time for the day is displayed.
        let total_time = "Total: 1h 30m";
        assert!(time_tracker.stdout.contains(total_time));

        // check that the time entries are displayed in reverse chronological order, with the most recent time entry at the top.
        let time_entry1 = "1. 01:00 - Test Entry 1 (Test Project 1: Test Task 1)";
        assert!(time_tracker.stdout.contains(time_entry1));
        let time_entry2 = "2. 01:30 - Test Entry 2 (Test Project 1: Test Task 2)";
        assert!(time_tracker.stdout.contains(time_entry2));

        // check that the sub-menu is displayed.
        let sub_menu = "New (n) | Edit (e) | Delete (d) | Change Date (c) | Go to Today (g)";
        assert!(time_tracker.stdout.contains(sub_menu));
    }
}
