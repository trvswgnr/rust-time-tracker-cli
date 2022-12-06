# Simple CLI Time Tracker built with Rust

This is a simple CLI time tracker built with Rust. It is a work in progress.

## Usage

```bash
$ time-tracker
```

## Installation

```bash
$ cargo install time-tracker
```

## License

MIT


## Feature Roadmap
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
