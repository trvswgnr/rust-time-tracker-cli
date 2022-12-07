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
  - [ ] Settings:
    - [ ] Allow the user to enter their name.
    - [ ] Allow the user to enter their email address.
- [ ] Start the program on the Time page, showing the current day.
- [ ] Store and retrieve data from an SQLite database.

Here are the libraries that are needed by this project:
- [ ] [clap](https://crates.io/crates/clap)
- [ ] [chrono](https://crates.io/crates/chrono)
- [ ] [crossterm](https://crates.io/crates/crossterm)
- [ ] [r2d2](https://crates.io/crates/r2d2)
- [ ] [r2d2_sqlite](https://crates.io/crates/r2d2_sqlite)
- [ ] [rusqlite](https://crates.io/crates/rusqlite)
- [ ] [serde](https://crates.io/crates/serde)
- [ ] [serde_json](https://crates.io/crates/serde_json)
- [ ] [serde_yaml](https://crates.io/crates/serde_yaml)
- [ ] [structopt](https://crates.io/crates/structopt)
- [ ] [termion](https://crates.io/crates/termion)
*/
#![allow(unused_imports, deprecated)]
use std::collections::hash_map::Entry;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::collections::VecDeque;
use std::error::Error;
use std::io::{self, Write};
use std::process;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::Instant;

use chrono;
use chrono::prelude::*;
use chrono::Duration as ChronoDuration;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyEventState;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute, queue, style};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use structopt::StructOpt;

// mod app;
// mod database;
// mod models;
// mod pages;
// mod time;
// mod utils;

#[derive(Clone)]
struct Paragraph<'a> {
    block: Option<Block<'a>>,
    text: Vec<Spans<'a>>,
    wrap: bool,
    alignment: Alignment,
    style: Style,
}

impl<'a> Paragraph<'a> {
    pub fn new(text: &'a str) -> Paragraph<'a> {
        Paragraph {
            block: None,
            text: vec![Spans::from(text)],
            wrap: true,
            alignment: Alignment::Left,
            style: Style::default(),
        }
    }

    pub fn block(&mut self, block: &mut Block<'a>) -> Paragraph<'a> {
        self.block = Some(block.clone());
        self.clone()
    }

    pub fn as_str(&self) -> &str {
        self.text[0].as_str()
    }
}

#[derive(Clone)]
enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Clone)]
struct Block<'a> {
    title: Option<Spans<'a>>,
    style: Style,
    border_type: BorderType,
    border_style: Style,
    borders: Borders,
    margin: Rect,
    width: u16,
    height: u16,
}

impl Block<'_> {
    pub fn new() -> Block<'static> {
        Block::default()
    }

    fn borders(&mut self, borders: Borders) -> &mut Self {
        self.borders = borders;
        self
    }
}

#[derive(Clone)]
enum Borders {
    NONE,
    TOP,
    BOTTOM,
    LEFT,
    RIGHT,
    TopBottom,
    LeftRight,
    ALL,
}

impl Default for Block<'_> {
    fn default() -> Block<'static> {
        Block {
            title: None,
            style: Style::default(),
            border_type: BorderType::Plain,
            border_style: Style::default(),
            borders: Borders::NONE,
            margin: Rect::default(),
            width: 0,
            height: 0,
        }
    }
}

#[derive(Clone)]
enum BorderType {
    Plain,
    Rounded,
    Thick,
    Double,
}

#[derive(Clone)]
struct Spans<'a> {
    spans: Vec<Span<'a>>,
}

impl Spans<'_> {
    pub fn as_str(&self) -> &str {
        self.spans[0].content
    }
}

impl<'a> From<&'a str> for Spans<'a> {
    fn from(s: &'a str) -> Spans<'a> {
        Spans {
            spans: vec![Span::from(s)],
        }
    }
}

impl core::fmt::Display for Spans<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for span in &self.spans {
            write!(f, "{}", span.content)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct Span<'a> {
    content: &'a str,
    style: Style,
}

impl<'a> From<&'a str> for Span<'a> {
    fn from(s: &'a str) -> Span<'a> {
        Span {
            content: s,
            style: Style::default(),
        }
    }
}

#[derive(Clone)]
struct Style {
    fg: Option<Color>,
    bg: Option<Color>,
    add_modifier: Modifier,
    sub_modifier: Modifier,
}

impl Default for Style {
    fn default() -> Style {
        Style {
            fg: None,
            bg: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
    }
}

#[derive(Clone)]
struct Modifier {
    bits: u8,
}

impl Modifier {
    const fn empty() -> Modifier {
        Modifier { bits: 0 }
    }
}

/// Holds the command line options
#[derive(StructOpt, Debug)]
#[structopt(name = "time-tracker", about = "A time tracking application")]
struct Opt {
    /// The database file to use
    #[structopt(short, long, parse(from_os_str))]
    database: PathBuf,
}

// then we implement our models
#[derive(Debug)]
struct Project {
    id: i64,
    name: String,
    description: String,
}

#[derive(Debug)]
struct Task {
    id: i64,
    project_id: i64,
    name: String,
    description: String,
}

#[derive(Clone, Debug)]
struct TimeEntry {
    /// The id of the time entry
    id: i64,
    /// The id of the associated project
    project_id: i64,
    /// The id of the associated task
    task_id: i64,
    /// The description of the time entry
    description: String,
    /// The start time (as a unix timestamp)
    start_time: i64,
    /// The end time (as a unix timestamp)
    end_time: i64,
    /// The duration in seconds
    duration: i64,
    /// The date the time entry was created
    /// (as a unix timestamp)
    created_at: i64,
}

trait ToDateTime {
    fn to_datetime(&self) -> DateTime<Utc>;
}

trait ToI64 {
    fn to_i64(&self) -> i64;
}

impl ToDateTime for i64 {
    fn to_datetime(&self) -> DateTime<Utc> {
        let duration = Duration::from_secs(*self as u64);
        // convert to a chrono::DateTime
        let duration = ChronoDuration::from_std(duration).unwrap();
        let datetime = Utc.timestamp(0, 0) + duration;
        datetime
    }
}

impl ToI64 for DateTime<Utc> {
    fn to_i64(&self) -> i64 {
        let duration = self.signed_duration_since(Utc.timestamp(0, 0));
        let duration = duration.to_std().unwrap().as_secs();
        duration as i64
    }
}

impl TimeEntry {
    fn new(
        id: i64,
        project_id: i64,
        task_id: i64,
        description: String,
        start_time: i64,
        end_time: i64,
        duration: i64,
    ) -> Self {
        TimeEntry {
            id,
            project_id,
            task_id,
            description,
            start_time,
            end_time,
            duration,
            created_at: Utc::now().to_i64(),
        }
    }

    fn duration(&self) -> i64 {
        self.end_time - self.start_time
    }

    fn duration_string(&self) -> String {
        let duration = self.duration();
        let hours = duration / 3600;
        let minutes = (duration % 3600) / 60;
        let seconds = duration % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    fn copy(&self) -> Self {
        TimeEntry {
            id: self.id,
            project_id: self.project_id,
            task_id: self.task_id,
            description: self.description.clone(),
            start_time: self.start_time,
            end_time: self.end_time,
            duration: self.duration,
            created_at: self.created_at,
        }
    }
}

impl Default for TimeEntry {
    fn default() -> Self {
        TimeEntry {
            id: 0,
            project_id: 0,
            task_id: 0,
            description: String::from("Time Entry"),
            start_time: 0,
            end_time: 0,
            duration: 0,
            created_at: Utc::now().to_i64(),
        }
    }
}

/// The page enum defines the different pages that the application can be on.
///
/// #### Parent Pages
/// * Time
/// * Projects
/// * Settings
///
/// #### Child Pages
/// * ProjectDetail (child of Projects)
/// * TimeEntryCreateOrEdit (child of Time)
/// * ProjectCreateOrEdit (child of Projects)
/// * TaskCreateOrEdit (child of ProjectDetail)
///
/// #### Page Flow
/// The child pages are only accessible from the parent pages.
/// The Time page is the default page.
/// While on any page, the user can press the 'q' key to quit the application, the 't' key to go to the Time page, the 'p' key to go to the Projects page, and the 's' key to go to the Settings page.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Pages {
    /// The time page shows the current week as a row of days with the total time for each day. It highlights the selected day. It defaults to the current day, showing the day's time entries. It allows the user to select a different day and a different week.
    Time,

    /// The projects page shows a list of projects. It allows the user to create a new project, edit a project, and delete a project. It allows the user to select a project, entering a project detail page.
    Projects,

    /// The settings page allows the user to enter their name and email address.
    Settings,

    /// The project detail page shows a list of tasks for the project. It allows the user to create a new task, edit a task, and delete a task.
    ProjectDetail,

    /// The time entry create or edit page allows the user to create a new time entry or edit an existing time entry.
    TimeEntryCreateOrEdit,

    /// The project create or edit page allows the user to create a new project or edit an existing project.
    ProjectCreateOrEdit,

    /// The task create or edit page allows the user to create a new task or edit an existing task.
    TaskCreateOrEdit,
}

struct Page {
    page: Pages,
    parent: Option<Pages>,
}

impl Page {
    fn from(page: Pages, parent: Option<Pages>) -> Page {
        Page { page, parent }
    }
}

macro_rules! get_key_event {
    ($key:expr) => {
        Event::Key(KeyEvent {
            code: KeyCode::Char($key),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    };
}

#[derive(Clone)]
struct Rect {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl Default for Rect {
    fn default() -> Self {
        Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }
}

struct Terminal {
    stdout: io::Stdout,
    stdin: io::Stdin,
    events: Receiver<Event>,
    running: Arc<AtomicBool>,
}

impl Terminal {
    fn new() -> Terminal {
        let (tx, rx) = channel();
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        thread::spawn(move || {
            let mut last_event = Instant::now();

            while r.load(Ordering::Relaxed) {
                if event::poll(Duration::from_millis(250)).unwrap() {
                    if let Ok(event) = event::read() {
                        if event == get_key_event!('q') {
                            r.store(false, Ordering::Relaxed);
                        }

                        if Instant::now().duration_since(last_event).as_millis() > 100 {
                            tx.send(event).unwrap();
                            last_event = Instant::now();
                        }
                    }
                }
            }
        });

        Terminal {
            stdout: io::stdout(),
            stdin: io::stdin(),
            events: rx,
            running,
        }
    }

    fn get_event(&self) -> crossterm::Result<Event> {
        match self.events.recv() {
            Ok(event) => Ok(event),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::Other,
                "Error receiving event",
            )),
        }
    }

    fn clear(&mut self) -> crossterm::Result<()> {
        execute!(self.stdout, Clear(ClearType::All))?;
        Ok(())
    }

    fn flush(&mut self) -> crossterm::Result<()> {
        self.stdout.flush()?;
        Ok(())
    }

    fn set_cursor(&mut self, x: u16, y: u16) -> crossterm::Result<()> {
        execute!(self.stdout, cursor::MoveTo(x, y))?;
        Ok(())
    }

    fn print(&mut self, text: &str) -> crossterm::Result<()> {
        execute!(self.stdout, Print(text))?;
        Ok(())
    }

    fn get_size(&self) -> crossterm::Result<Rect> {
        let size = terminal::size()?;

        Ok(Rect {
            x: 0,
            y: 0,
            width: size.0,
            height: size.1,
        })
    }

    fn read_line(&mut self) -> crossterm::Result<String> {
        let mut input = String::new();
        self.stdin.read_line(&mut input)?;
        Ok(input)
    }
}

struct Screen {
    terminal: Terminal,
    size: Rect,
}

impl Screen {
    fn new() -> Screen {
        let mut terminal = Terminal::new();
        let term_size = terminal::size().unwrap();

        Screen {
            terminal,
            size: Rect {
                x: 0,
                y: 0,
                width: term_size.0,
                height: term_size.1,
            },
        }
    }

    fn get_event(&self) -> crossterm::Result<Event> {
        return self.terminal.get_event();
    }

    fn clear(&mut self) -> crossterm::Result<()> {
        return self.terminal.clear();
    }

    fn flush(&mut self) -> crossterm::Result<()> {
        return self.terminal.flush();
    }

    fn set_cursor(&mut self, x: u16, y: u16) -> crossterm::Result<()> {
        return self.terminal.set_cursor(x, y);
    }

    fn render(&mut self, text: &str) -> crossterm::Result<()> {
        return self.terminal.print(text);
    }

    fn read_line(&mut self) -> crossterm::Result<String> {
        return self.terminal.read_line();
    }

    fn draw(&mut self, text: &str) -> Result<(), Box<dyn Error>> {
        self.write_all(text.as_bytes())?;
        self.flush()?;
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Box<dyn Error>> {
        let mut written = 0;
        while written < buf.len() {
            let n = self.write(&buf[written..])?;
            if n == 0 {
                return Err("failed to write whole buffer".into());
            }
            written += n;
        }
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, Box<dyn Error>> {
        let mut stdout = io::stdout();
        stdout.write(buf)?;
        stdout.flush()?;
        Ok(buf.len())
    }
}

struct Menu {
    items: Vec<(String, String)>,
}

impl Menu {
    fn new() -> Self {
        Self { items: vec![] }
    }

    fn item(&mut self, text: &str, key: &str) {
        self.items.push((text.to_string(), key.to_string()));
    }

    fn render(&self) -> String {
        let mut menu = String::new();
        for (text, key) in &self.items {
            menu.push_str(&format!("{} ({}) ", text, key));
        }
        menu
    }
}

struct Settings {
    database_path: PathBuf,
    user: User,
}

struct User {
    name: String,
    email: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("timetracker.sqlite"),
            user: User {
                name: String::from("John Doe"),
                email: String::from("johndoe@example.com"),
            },
        }
    }
}

// then we implement our database
struct Database {
    connection: Connection,
}

impl Database {
    /// Creates a new database instance.
    /// If the database file does not exist, it will be created.
    fn new(path: &Path) -> Result<Database, Box<dyn Error>> {
        let connection = Connection::open(path)?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                  id              INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL,
                  description     TEXT NOT NULL
                  )",
            NO_PARAMS,
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                  id              INTEGER PRIMARY KEY,
                  project_id      INTEGER NOT NULL,
                  name            TEXT NOT NULL,
                  description     TEXT NOT NULL
                  )",
            NO_PARAMS,
        )?;

        /*
        we add a new table for time entries
        id: i64,
        project_id: i64,
        task_id: i64,
        description: String,
        start_time: i64,
        end_time: i64,
        duration: i64,
        created_at: i64,
        */
        connection.execute(
            "CREATE TABLE IF NOT EXISTS time_entries (
                  id              INTEGER PRIMARY KEY,
                  project_id      INTEGER NOT NULL,
                  task_id         INTEGER NOT NULL,
                  description     TEXT NOT NULL,
                  start_time      INTEGER NOT NULL,
                  end_time        INTEGER NOT NULL,
                  duration        INTEGER NOT NULL,
                  created_at      INTEGER NOT NULL
                  )",
            NO_PARAMS,
        )?;

        // create a settings table
        connection.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                  id              INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL,
                  email           TEXT NOT NULL,
                  database_path   TEXT NOT NULL
                  )",
            NO_PARAMS,
        )?;
        Ok(Database { connection })
    }

    fn save_settings(&self, name: String, email: String) -> Result<(), Box<dyn Error>> {
        self.connection.execute(
            "INSERT INTO settings (name, email, database_path) VALUES (?1, ?2, ?3)",
            params![
                name,
                email,
                self.connection.path().unwrap().to_str().unwrap()
            ],
        )?;
        Ok(())
    }

    fn get_settings(&self) -> Result<Settings, Box<dyn Error>> {
        let mut stmt = self
            .connection
            .prepare("SELECT * FROM settings ORDER BY id DESC LIMIT 1")?;
        let settings_iter = stmt.query_map(NO_PARAMS, |row| {
            let database_path: String = row.get(3)?;
            Ok(Settings {
                database_path: PathBuf::from(database_path),
                user: User {
                    name: row.get(1)?,
                    email: row.get(2)?,
                },
            })
        })?;

        for settings in settings_iter {
            return Ok(settings?);
        }

        Ok(Settings::default())
    }

    fn create_project(&self, name: String, description: String) -> Result<Project, Box<dyn Error>> {
        self.connection.execute(
            "INSERT INTO projects (name, description) VALUES (?1, ?2)",
            params![name, description],
        )?;
        let id = self.connection.last_insert_rowid();
        Ok(Project {
            id,
            name: name.to_string(),
            description: description.to_string(),
        })
    }

    fn get_project(&self, id: i64) -> Result<Project, Box<dyn Error>> {
        let mut stmt = self
            .connection
            .prepare("SELECT * FROM projects WHERE id = ?1")?;
        let project_iter = stmt.query_map(params![id], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })?;

        for project in project_iter {
            return Ok(project?);
        }

        Err(From::from("Project not found"))
    }

    fn get_all_projects(&self) -> Result<Vec<Project>, Box<dyn Error>> {
        let mut stmt = self.connection.prepare("SELECT * FROM projects")?;
        let project_iter = stmt.query_map(NO_PARAMS, |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })?;

        let mut projects = Vec::new();
        for project in project_iter {
            projects.push(project?);
        }

        Ok(projects)
    }

    fn create_task(
        &self,
        project_id: i64,
        name: &str,
        description: &str,
    ) -> Result<Task, Box<dyn Error>> {
        self.connection.execute(
            "INSERT INTO tasks (project_id, name, description) VALUES (?1, ?2, ?3)",
            params![project_id, name, description],
        )?;
        let id = self.connection.last_insert_rowid();
        Ok(Task {
            id,
            project_id,
            name: name.to_string(),
            description: description.to_string(),
        })
    }

    fn get_task(&self, id: i64) -> Result<Task, Box<dyn Error>> {
        let mut stmt = self
            .connection
            .prepare("SELECT * FROM tasks WHERE id = ?1")?;
        let task_iter = stmt.query_map(params![id], |row| {
            Ok(Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
            })
        })?;

        for task in task_iter {
            return Ok(task?);
        }

        Err(From::from("Task not found"))
    }

    fn get_tasks_for_project(&self, project_id: i64) -> Result<Vec<Task>, Box<dyn Error>> {
        let mut stmt = self
            .connection
            .prepare("SELECT * FROM tasks WHERE project_id = ?1")?;
        let task_iter = stmt.query_map(params![project_id], |row| {
            Ok(Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
            })
        })?;

        let mut tasks = Vec::new();
        for task in task_iter {
            tasks.push(task?);
        }

        Ok(tasks)
    }

    fn get_all_tasks(&self) -> Result<Vec<Task>, Box<dyn Error>> {
        let mut stmt = self.connection.prepare("SELECT * FROM tasks")?;
        let task_iter = stmt.query_map(NO_PARAMS, |row| {
            Ok(Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
            })
        })?;

        let mut tasks = Vec::new();
        for task in task_iter {
            tasks.push(task?);
        }

        Ok(tasks)
    }

    fn create_time_entry(
        &self,
        project_id: i64,
        task_id: i64,
        description: &str,
        start_time: i64,
        end_time: i64,
        duration: i64,
        created_at: i64,
    ) -> Result<TimeEntry, Box<dyn Error>> {
        self.connection.execute(
            "INSERT INTO time_entries (project_id, task_id, description, start_time, end_time, duration, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![project_id, task_id, description, start_time, end_time],
        )?;
        let id = self.connection.last_insert_rowid();
        Ok(TimeEntry {
            id,
            project_id,
            task_id,
            description: description.to_string(),
            start_time,
            end_time,
            duration,
            created_at,
        })
    }

    fn get_time_entry(&self, id: i64) -> Result<TimeEntry, Box<dyn Error>> {
        let mut stmt = self
            .connection
            .prepare("SELECT * FROM time_entries WHERE id = ?1")?;
        let time_entry_iter = stmt.query_map(params![id], |row| {
            Ok(TimeEntry {
                id: row.get(0)?,
                project_id: row.get(1)?,
                task_id: row.get(2)?,
                description: row.get(3)?,
                start_time: row.get(4)?,
                end_time: row.get(5)?,
                duration: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;

        for time_entry in time_entry_iter {
            return Ok(time_entry?);
        }

        Err(From::from("Time entry not found"))
    }

    fn get_time_entries_for_task(&self, task_id: i64) -> Result<Vec<TimeEntry>, Box<dyn Error>> {
        let mut stmt = self
            .connection
            .prepare("SELECT * FROM time_entries WHERE task_id = ?1")?;
        let time_entry_iter = stmt.query_map(params![task_id], |row| {
            Ok(TimeEntry {
                id: row.get(0)?,
                project_id: row.get(1)?,
                task_id: row.get(2)?,
                description: row.get(3)?,
                start_time: row.get(4)?,
                end_time: row.get(5)?,
                duration: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;

        let mut time_entries = Vec::new();
        for time_entry in time_entry_iter {
            time_entries.push(time_entry?);
        }

        Ok(time_entries)
    }

    fn get_all_time_entries(&self) -> Result<Vec<TimeEntry>, Box<dyn Error>> {
        let mut stmt = self.connection.prepare("SELECT * FROM time_entries")?;
        let time_entry_iter = stmt.query_map(NO_PARAMS, |row| {
            Ok(TimeEntry {
                id: row.get(0)?,
                project_id: row.get(1)?,
                task_id: row.get(2)?,
                description: row.get(3)?,
                start_time: row.get(4)?,
                end_time: row.get(5)?,
                duration: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;

        let mut time_entries = Vec::new();
        for time_entry in time_entry_iter {
            time_entries.push(time_entry?);
        }

        Ok(time_entries)
    }

    fn get_time_entries_for_date(&self, date: i64) -> Result<Vec<TimeEntry>, Box<dyn Error>> {
        let mut stmt = self
            .connection
            .prepare("SELECT * FROM time_entries WHERE start_time >= ?1 AND start_time < ?2")?;
        let time_entry_iter = stmt.query_map(params![date, date + 86400], |row| {
            Ok(TimeEntry {
                id: row.get(0)?,
                project_id: row.get(1)?,
                task_id: row.get(2)?,
                description: row.get(3)?,
                start_time: row.get(4)?,
                end_time: row.get(5)?,
                duration: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;

        let mut time_entries = Vec::new();
        for time_entry in time_entry_iter {
            time_entries.push(time_entry?);
        }

        Ok(time_entries)
    }

    fn update_time_entry(
        &self,
        id: i64,
        project_id: i64,
        task_id: i64,
        description: &str,
        start_time: i64,
        end_time: i64,
        duration: i64,
        created_at: i64,
    ) -> Result<(), Box<dyn Error>> {
        self.connection.execute(
            "UPDATE time_entries SET project_id = ?1, task_id = ?2, description = ?3, start_time = ?4, end_time = ?5, duration = ?6, created_at = ?7 WHERE id = ?8",
            params![project_id, task_id, description, start_time, end_time, duration, created_at, id],
        )?;
        Ok(())
    }

    fn delete_time_entry(&self, id: i64) -> Result<(), Box<dyn Error>> {
        self.connection
            .execute("DELETE FROM time_entries WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn delete_project(&self, id: i64) -> Result<(), Box<dyn Error>> {
        self.connection
            .execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(())
    }
}

// then we implement our application
pub struct App {
    database: Database,
    page_stack: Vec<Pages>,
    selection: usize,
    time_entries: Vec<TimeEntry>,
    projects: Vec<Project>,
    tasks: Vec<Task>,
    time_entry: Option<TimeEntry>,
    project: Option<Project>,
    task: Option<Task>,
    settings: Settings,
    week: i64,
    day: i64,
    quit: bool,
    screen: Screen,
    running: Arc<AtomicBool>,
}

impl App {
    pub fn new() -> App {
        let settings = Settings::default();
        let db_path = std::path::Path::new(&settings.database_path);
        let database = Database::new(db_path).unwrap();

        App {
            database,
            page_stack: vec![Pages::Time],
            selection: 0,
            time_entries: vec![],
            projects: vec![],
            tasks: vec![],
            time_entry: None,
            project: None,
            task: None,
            settings,
            week: 0,
            day: 0,
            quit: false,
            screen: Screen::new(),
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.load_data()?;
        self.render()?;
        Ok(())
    }

    fn load_data(&mut self) -> Result<(), Box<dyn Error>> {
        self.time_entries = self.database.get_all_time_entries()?;
        self.projects = self.database.get_all_projects()?;
        self.tasks = self.database.get_all_tasks()?;
        // get the name and email from the settings
        self.settings = self.database.get_settings()?;

        Ok(())
    }

    fn render(&mut self) -> Result<(), Box<dyn Error>> {
        let page = self.page_stack.last().unwrap();
        self.screen.clear()?;
        match page {
            Pages::Time => self.render_time_entries()?,
            Pages::Projects => self.render_projects()?,
            Pages::Settings => self.render_settings()?,
            Pages::ProjectDetail => self.render_project_detail()?,
            Pages::TimeEntryCreateOrEdit => self.render_time_entry_create_or_edit()?,
            Pages::ProjectCreateOrEdit => self.render_project_create_or_edit()?,
            Pages::TaskCreateOrEdit => self.render_task_create_or_edit()?,
        }
        self.screen.flush()?;

        self.handle_events()?;
        Ok(())
    }

    /// Handle events
    /// This is where we handle user input
    /// We use the `crossterm` crate to handle input
    fn handle_events(&mut self) -> Result<(), Box<dyn Error>> {
        let event = event::read()?;
        match event {
            Event::Key(key_event) => {
                let page = self.page_stack.last().unwrap();
                match page {
                    Pages::Time => self.handle_page_event(Pages::Time, key_event)?,
                    Pages::Projects => self.handle_page_event(Pages::Projects, key_event)?,
                    Pages::Settings => self.handle_page_event(Pages::Settings, key_event)?,
                    Pages::ProjectDetail => {
                        self.handle_page_event(Pages::ProjectDetail, key_event)?
                    }
                    Pages::TimeEntryCreateOrEdit => {
                        self.handle_page_event(Pages::TimeEntryCreateOrEdit, key_event)?
                    }
                    Pages::ProjectCreateOrEdit => {
                        self.handle_page_event(Pages::ProjectCreateOrEdit, key_event)?
                    }
                    Pages::TaskCreateOrEdit => {
                        self.handle_page_event(Pages::TaskCreateOrEdit, key_event)?
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Render the time entries
    /// This is where we render the time entries
    /// We render the time entries in a list for the current day
    /// We also render the total time for the day
    fn render_time_entries(&mut self) -> Result<(), Box<dyn Error>> {
        let mut time_entries = vec![];
        for time_entry in &self.time_entries {
            time_entries.push(format!(
                "{} - {} - {}",
                time_entry.start_time, time_entry.end_time, time_entry.description
            ));
        }
        let mut selection = self.selection;
        if time_entries.is_empty() {
            selection = 0;
        } else {
            if selection >= time_entries.len() {
                selection = time_entries.len() - 1;
            }
        }
        let selection = selection as u16;
        let mut menu = Menu::new();
        menu.item("Create time entry", "c");
        menu.item("Edit time entry", "e");
        menu.item("Delete time entry", "d");
        menu.item("Projects", "p");
        menu.item("Settings", "s");
        menu.item("Quit", "q");
        let menu = menu.render();
        let mut time_entries = time_entries.join("\n");
        if time_entries.is_empty() {
            time_entries = "No time entries".to_string();
        }

        let mut total_time = 0;
        for time_entry in &self.time_entries {
            total_time += time_entry.duration();
        }

        let total_time = format!("Total time: {}", total_time);

        let mut time_entries = vec![menu, time_entries, total_time];
        time_entries.insert(0, "Time entries".to_string());
        let time_entries = time_entries.join("\n");
        let mut time_entries = Paragraph::new(&time_entries);
        time_entries = time_entries.block(Block::default().borders(Borders::ALL));
        self.screen.render(time_entries.as_str())?;
        self.screen.set_cursor(0, selection)?;
        Ok(())
    }

    fn render_projects(&mut self) -> Result<(), Box<dyn Error>> {
        let mut projects = vec![];
        for project in &self.projects {
            projects.push(format!("{} - {}", project.name, project.description));
        }
        let mut selection = self.selection;
        if projects.is_empty() {
            selection = 0;
        } else {
            if selection >= projects.len() {
                selection = projects.len() - 1;
            }
        }
        let selection = selection as u16;
        let mut menu = Menu::new();
        menu.item("Create project", "c");
        menu.item("Edit project", "e");
        menu.item("Delete project", "d");
        menu.item("Time", "t");
        menu.item("Settings", "s");
        menu.item("Quit", "q");
        let menu = menu.render();
        let mut projects = projects.join(
            "
",
        );

        projects.push_str(&menu);
        self.screen.clear()?;
        self.screen.draw(&projects)?;
        self.screen.set_cursor(0, selection)?;
        Ok(())
    }

    fn render_settings(&mut self) -> Result<(), Box<dyn Error>> {
        let mut menu = Menu::new();
        menu.item("Time", "t");
        menu.item("Projects", "p");
        menu.item("Quit", "q");
        let menu = menu.render();

        // screen: RawTerminal<io::Stdout>
        self.screen.flush()?;
        self.screen.clear()?;
        self.screen.draw(&menu)?;
        Ok(())
    }

    fn render_project_detail(&mut self) -> Result<(), Box<dyn Error>> {
        let project = self
            .projects
            .iter()
            .find(|project| project.id == self.selection as i64)
            .unwrap();
        let mut tasks = vec![];
        for task in &self.tasks {
            if task.project_id == project.id {
                tasks.push(format!("{} - {}", task.name, task.description));
            }
        }
        let mut selection = self.selection;
        if selection >= tasks.len() {
            selection = tasks.len() - 1;
        }
        let selection = selection as u16;
        let mut menu = Menu::new();
        menu.item("Create task", "c");
        menu.item("Edit task", "e");
        menu.item("Delete task", "d");
        menu.item("Back", "b");
        let menu = menu.render();
        let mut tasks = tasks.join(
            "
",
        );

        tasks.push_str(&menu);
        self.screen.clear()?;
        self.screen.draw(&tasks)?;
        self.screen.set_cursor(0, selection)?;
        Ok(())
    }

    fn render_time_entry_create_or_edit(&mut self) -> Result<(), Box<dyn Error>> {
        let mut menu = Menu::new();
        menu.item("Save", "s");
        menu.item("Back", "b");
        let menu = menu.render();
        self.screen.clear()?;
        self.screen.draw(&menu)?;
        Ok(())
    }

    /// Render the project create or edit screen
    ///
    /// This will allow the user to create a new project or edit an existing project.
    /// The user will be prompted to enter a project name and description.
    /// The project will be saved to the database and added to the `projects` vector.
    /// The user will be returned to the project list screen, which will be rendered and the project will be selected.
    fn render_project_create_or_edit(&mut self) -> Result<(), Box<dyn Error>> {
        let mut menu = Menu::new();
        menu.item("Save", "s");
        menu.item("Back", "b");
        let menu = menu.render();
        self.screen.clear()?;
        self.screen.draw(&menu)?;

        self.screen.draw("Enter a project name: ")?;
        let project_name = self.screen.read_line()?;

        self.screen.draw("Enter a project description: ")?;
        let project_description = self.screen.read_line()?;

        // create a new project
        let project = self
            .database
            .create_project(project_name, project_description)?;

        // add the project to the projects vector
        self.projects.push(project);

        // select the project
        self.selection = self.projects.len() - 1;

        // return to the project list screen
        self.page_stack.pop();
        self.render()?;
        Ok(())
    }

    /// Render the task create or edit screen
    /// This will allow the user to create a new task or edit an existing task
    /// The user will be prompted to enter a task name and description
    /// The user will be returned to the project detail screen
    ///
    fn render_task_create_or_edit(&mut self) -> Result<(), Box<dyn Error>> {
        let mut menu = Menu::new();
        menu.item("Save", "s");
        menu.item("Back", "b");
        let menu = menu.render();

        self.screen.draw("Enter a task name: ")?;
        let task_name = self.screen.read_line()?;

        self.screen.draw("Enter a task description: ")?;

        let task_description = self.screen.read_line()?;

        // create a new task
        let task = self.database.create_task(
            self.project.as_mut().unwrap().id,
            &task_name,
            &task_description,
        )?;

        // add the task to the tasks vector
        self.tasks.push(task);

        self.screen.clear()?;
        self.screen.draw(&menu)?;
        Ok(())
    }

    fn delete_time_entry(&mut self) -> Result<(), Box<dyn Error>> {
        let time_entry = self
            .time_entries
            .iter()
            .find(|time_entry| time_entry.id == self.selection as i64)
            .unwrap();
        let time_entry_id = time_entry.id;
        self.time_entries
            .retain(|time_entry| time_entry.id != time_entry_id);

        self.database.delete_time_entry(time_entry_id)?;
        Ok(())
    }

    fn delete_project(&mut self) -> Result<(), Box<dyn Error>> {
        let project = self
            .projects
            .iter()
            .find(|project| project.id == self.selection as i64)
            .unwrap();
        let project_id = project.id;
        self.projects.retain(|project| project.id != project_id);

        self.database.delete_project(project_id)?;
        Ok(())
    }

    fn handle_time_page_event(&mut self, key_event: KeyEvent) -> Result<(), Box<dyn Error>> {
        // use the page_stack to determine what page we are on
        // if we are on the time page, then we can handle the event
        let page = self.page_stack.last().unwrap();
        if page == &Pages::Time {
            match key_event.code {
                KeyCode::Char('c') => {
                    self.page_stack.push(Pages::TimeEntryCreateOrEdit);
                }
                KeyCode::Char('e') => {
                    self.page_stack.push(Pages::TimeEntryCreateOrEdit);
                }
                KeyCode::Char('d') => {
                    self.delete_time_entry()?;
                }
                KeyCode::Char('p') => {
                    self.page_stack.push(Pages::Projects);
                }
                KeyCode::Char('s') => {
                    self.page_stack.push(Pages::Settings);
                }
                KeyCode::Char('q') => {
                    self.quit = true;
                }
                _ => {}
            }
        }

        self.render()?;

        return Ok(());
    }

    fn handle_project_page_event(&mut self, key_event: KeyEvent) -> Result<(), Box<dyn Error>> {
        // use the page_stack to determine what page we are on
        // if we are on the project page, then we can handle the event
        let page = self.page_stack.last().unwrap();
        if page == &Pages::Projects {
            match key_event.code {
                // Create a new project
                KeyCode::Char('c') => {
                    self.page_stack.push(Pages::ProjectCreateOrEdit);
                }

                // Edit the project
                KeyCode::Char('e') => {
                    self.page_stack.push(Pages::ProjectCreateOrEdit);
                }

                // Delete the project
                KeyCode::Char('d') => {
                    self.delete_project()?;
                }

                // Go to the time page
                KeyCode::Char('t') => {
                    self.page_stack.push(Pages::ProjectDetail);
                }

                // Go back to the previous page
                KeyCode::Char('b') => {
                    self.page_stack.pop();
                }

                // Go to the settings page
                KeyCode::Char('s') => {
                    self.page_stack.push(Pages::Settings);
                }

                // Quit the application
                KeyCode::Char('q') => {
                    self.quit = true;
                }
                _ => {}
            }
        }

        self.render()?;

        return Ok(());
    }

    fn handle_settings_page_event(&mut self, key_event: KeyEvent) -> Result<(), Box<dyn Error>> {
        // use the page_stack to determine what page we are on
        // if we are on the settings page, then we can handle the event
        let page = self.page_stack.last().unwrap();
        if page == &Pages::Settings {
            match key_event.code {
                // Go back to the previous page
                KeyCode::Char('b') => {
                    self.page_stack.pop();
                }

                // Quit the application
                KeyCode::Char('q') => {
                    self.quit = true;
                }
                _ => {}
            }
        }

        self.render()?;

        return Ok(());
    }

    /// Handle time entry create or edit page events
    ///
    /// This function handles the events for the time entry create or edit page.
    ///
    /// The user will be presented with the following options:
    /// - Edit the time entry description
    /// - Edit the time entry duration
    /// - Start the timer for the time entry
    /// - Stop the timer for the time entry
    /// - Save the time entry
    /// - Delete the time entry
    /// - Go back to the previous page
    fn handle_time_entry_create_or_edit_page_event(
        &mut self,
        key_event: KeyEvent,
    ) -> Result<(), Box<dyn Error>> {
        // use the page_stack to determine what page we are on
        // if we are on the time entry create or edit page, then we can handle the event
        let page = self.page_stack.last().unwrap();
        // if we have a time entry selected, then we are editing it.
        let mut editing = false;
        // determine if we are editing or creating a time entry
        if self.selection != 0 {
            editing = true;
        }
        let (mut tx, mut rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        if page == &Pages::TimeEntryCreateOrEdit {
            match key_event.code {
                // Edit the time entry description
                KeyCode::Char('e') => {
                    // get the description from the user
                    self.screen.draw("Enter the time entry description: ")?;
                    let desc = self.screen.read_line()?;
                    // send the description to the receiver
                    tx.send("_description: ".to_string() + &desc).unwrap();
                }
                // Edit the time entry duration
                KeyCode::Char('t') => {
                    // get the duration from the user
                    self.screen
                        .draw("Enter the time entry duration hours(e.g. 4.5): ")?;
                    let hours_str = self.screen.read_line()?;
                    // convert the string to a number
                    let hours = hours_str.parse::<f32>()?;
                    // convert hours to seconds
                    let seconds = hours * 3600.0;

                    // convert the duration to a string
                    // send the duration to the receiver
                    tx.send("_duration: ".to_string() + &seconds.to_string())
                        .unwrap();
                }

                // Start the timer for the time entry
                KeyCode::Char('s') => {
                    while KeyCode::Char(' ') != key_event.code {
                        // get the current time
                        // let now = chrono::Local::now();
                        // // convert `now` to an i64
                        // let now = now.timestamp();
                        // // get the duration from the start time
                        // let duration = now - self.time_entry.unwrap().start_time;
                        // // get the duration in seconds
                        // let seconds = duration;
                        // // convert the seconds to hours
                        // let hours = seconds as f32 / 3600.0;
                        // // format the hours to 2 decimal places
                        // let hours_str = format!("{:.2}", hours);
                        // update the duration
                        // self.time_entry.duration = hours_str;
                        // render the page
                        // self.render()?;
                        // sleep for 1 second
                        println!("sleeping");
                        thread::sleep(Duration::from_secs(1));
                    }
                }

                // Stop the timer for the time entry (space bar)
                KeyCode::Char(' ') => {
                    // get the current time
                    // let now = chrono::Local::now();
                    // // convert `now` to an i64
                    // let now = now.timestamp();
                    // // get the duration from the start time
                    // let duration = now - self.time_entry.unwrap().start_time;
                    // // get the duration in seconds
                    // let seconds = duration;
                    // // convert the seconds to hours
                    // let hours = seconds as f32 / 3600.0;
                    // // format the hours to 2 decimal places
                    // let hours_str = format!("{:.2}", hours);
                    // update the duration
                    // self.time_entry.duration = hours_str;
                    // render the page
                    // self.render()?;
                }

                // Save the time entry
                KeyCode::Char('a') => {
                    // if we are editing, then update the time entry
                    if editing {
                        // update the time entry
                        // self.update_time_entry()?;
                    } else {
                        // create the time entry
                        // self.create_time_entry()?;
                    }
                }

                // Delete the time entry
                KeyCode::Char('d') => {
                    self.delete_time_entry()?;
                }

                // Go back to the previous page
                KeyCode::Char('b') => {
                    self.page_stack.pop();
                }

                // Quit the application
                KeyCode::Char('q') => {
                    self.quit = true;
                }
                _ => {}
            }
        }

        // check if we have a message from the receiver
        if let Ok(msg) = rx.try_recv() {
            // if we have a message, then update the time entry
            self.update_time_entry_from_message(msg)?;
        }

        self.render()?;

        return Ok(());
    }

    fn update_time_entry_from_message(&mut self, msg: String) -> Result<(), Box<dyn Error>> {
        // split the message into the field and value
        let parts: Vec<&str> = msg.split(": ").collect();
        // get the field
        let field = parts[0];
        // get the value
        let value = parts[1];

        // update the time entry
        match field {
            "_description" => self.time_entry.as_mut().unwrap().description = value.to_string(),
            "_duration" => self.time_entry.as_mut().unwrap().duration = value.parse::<i64>()?,
            _ => {}
        }

        let TimeEntry {
            id,
            project_id,
            description,
            duration,
            start_time,
            end_time,
            task_id,
            created_at,
        } = self.time_entry.as_ref().unwrap();

        // update the time entry in the database
        self.database.update_time_entry(
            *id,
            *project_id,
            *task_id,
            description,
            *start_time,
            *end_time,
            *duration,
            *created_at,
        )?;

        Ok(())
    }

    fn create_time_entry_from_message(&mut self, msg: String) -> Result<(), Box<dyn Error>> {
        // split the message into the field and value
        let parts: Vec<&str> = msg.split(": ").collect();
        // get the field
        let field = parts[0];
        // get the value
        let value = parts[1];

        // create the time entry
        let time_entry = TimeEntry::default();
        self.time_entry = Some(time_entry);
        match field {
            "_description" => self.time_entry.as_mut().unwrap().description = value.to_string(),
            "_duration" => self.time_entry.as_mut().unwrap().duration = value.parse::<i64>()?,
            _ => {}
        }

        // destucture the time entry
        let TimeEntry {
            project_id,
            description,
            duration,
            start_time,
            end_time,
            task_id,
            created_at,
            ..
        } = self.time_entry.as_ref().unwrap();

        // save the time entry to the database
        self.database.create_time_entry(
            *project_id,
            *task_id,
            description,
            *start_time,
            *end_time,
            *duration,
            *created_at,
        )?;

        self.selection = self.time_entry.as_ref().unwrap().id as usize;

        Ok(())
    }

    /// Handle page events
    fn handle_page_event(
        &mut self,
        page: Pages,
        key_event: KeyEvent,
    ) -> Result<(), Box<dyn Error>> {
        // handle quit (q)
        if key_event.code == KeyCode::Char('q') {
            self.running = Arc::new(AtomicBool::new(false));
            self.quit = true;
            return Ok(());
        }

        match page {
            Pages::Time => self.handle_time_page_event(key_event),
            Pages::Projects => self.handle_project_page_event(key_event),
            Pages::Settings => self.handle_settings_page_event(key_event),
            Pages::TimeEntryCreateOrEdit => {
                self.handle_time_entry_create_or_edit_page_event(key_event)
            }
            // Pages::ProjectCreateOrEdit => self.handle_project_create_or_edit_page_event(key_event),
            // Pages::TaskCreateOrEdit => self.handle_task_create_or_edit_page_event(key_event),
            _ => Ok(()),
        }
    }
}
