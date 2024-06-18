# Changelog

## Unreleased (on main branch only)

- Improve `reqwest` errors

## 2024-06-18 v0.6.10

- Fix `DateTime` sorting

- Divide page size (from term size in last commit) by two to fix prompt spacing bug.

## 2024-06-09 v0.6.9

- Add `list timebox`
- Adjust page size of select options based on terminal size

## 2024-05-30 v0.6.8

- Don't show spinner for `list label` asynchronous requests
- Add `Skip` option to `tod list label`
- Sort the tasks returned from `list label`
- Add configurable terminal bell with `bell_on_success` and `bell_on_failure`

## 2024-05-28 v0.6.7

- Put the error channel transmitter in `Config`, removing the need to pass around the additional argument in many places
- Add `config set-timezone`
- Format Markdown links as OSC8 links in supported terminals. Falls back in unsupported terminal
- Added `disable-links` configuration option

## 2024-05-26 v0.6.6

- Changed file system calls to asynchronous `tokio` calls
- Asynchronous errors are now sent to a channel, so they can be printed at the end rather than when the error occurs (which can disrupt the formatting of any menu that the user is currently in)
- Add `auto` to `project import`, which imports any projects not in config
- Add additional instructions to project missing error

## 2024-05-18 v0.6.5

- Internal refactoring to use an error struct and provide better errors
- Ask for labels in `list label` when labels are not provided. Previously just showed an error
- Check Cargo version asynchronously
- Make `list schedule` calls asynchronously
- Make `list label` calls asynchronously
- Make `list prioritize` calls asynchronously

## 2024-05-18 v0.6.4

- Remove blocking `reqwest` client

## 2024-05-12 v0.6.3

- Make `list process` calls asynchronously
- Make `project empty` calls asynchronously

## 2024-05-06 v0.6.2

- Make `task quick-add` `--content` flag optional (this feature was lost in the 0.6.0 API change)

## 2024-05-05 v0.6.1

- Show project name when printing a task when the task was obtained through a filter (not a project)
- Add `timeout` argument
- Add `repeat` flag to `project remove`

## 2024-05-04 v0.6.0

- **BREAKING CHANGE** Rewrote the external API, both to make it more coherent for the end user and to make it easier to add to from a development perspective. No functionality lost. The main change is that instead of using the `project` or `filter` command for searching for lists of tasks, we now use the `list` command that can take either `--project` or `--filter` flags. This is a change needed to proceed with planned features
- Added short commands for all sub commands and flags
- Switch back to Sync API for completing tasks, because REST API doesn't handle subtasks correctly
- Config path now supports expanding `~` into the home directory
- Add `config reset` command that deletes config

## 2024-03-28 v0.5.13

- Add spaces in between tasks in `process`
- Show JSON responses when using `verbose` flag

## 2024-03-28 v0.5.12

- Don't `process` child tasks when the parent task is in the future
- Don't `process` parent tasks when child tasks are still unchecked
- Disable spinner in test
- Use REST API for completing tasks

## 2024-03-26 v0.5.11

- Added more `quickadd` examples
- Don't `empty` subtasks
- Add delete option to `empty` and `process`
- Stopped overwriting verbose in config when using `verbose` flag
- Print task durations

## 2024-03-17 v0.5.10

- Allow custom sorting of tasks through config
- Fix bug that deleted projects in config rather than renaming them

## 2024-03-10 v0.5.9

- Add tasks remaining counter to `process` sub commands

## 2024-03-07 v0.5.8

- Improve timezone parsing to handle offsets

## 2023-11-05 v0.5.7

- Add `labels` argument to `task create`
- Fix reversed priorities on `task create`

## 2023-10-27 v0.5.6

- Improve formatting when labelling tasks
- Sort tasks by value when processing

## 2023-10-16 v0.5.5

- Fix `schedule` and `prioritize` (they were asking for projects rather than filters)

## 2023-10-16 v0.5.4

- Add labels to formatted tasks
- Improve task formatting
- Add `filter` flag to `task list`
- Remove `scheduled` flag from `task list`, the `filter` flag covers this use case now. Use `today & !no time`
- Add `filter` flag to `task edit` and `task next`
- Add `filter label`
- Add `filter schedule`
- Add `filter prioritize`

## 2023-10-09 v0.5.3

- Add `verbose` flag
- BREAKING CHANGE changed character flag for `overdue` from `v` to `u`
- Put single quotes around project names when printing them in terminal
- Re-enable `project schedule` for recurring tasks
- Add `skip-recurring` flag to `project schedule`
- Display due string when formatting task

## 2023-09-18 v0.5.2

- Add `nosection` flag to `task create`
- Add `No section` option to section selection
- Add `no_sections` config option

## 2023-09-04 v0.5.1

- Add details about configuration file
- Fix `spinners` config check
- Don't prompt for project when there is only one project in configuration file, just use it
- `task create` asks for section when project has sections
- Add a `natural_language_only` option to config

## 2023-07-30 v0.5.0

- Use structs for projects instead storing as a `HashMap` in config. This means that projects need to imported again with `project import`, sorry for the inconvenience. The tech debt around project handling was slowing down development
- Put date picker option first when scheduling
- Remove `project add` as only `project import` can be used now
- Rename Items to Tasks internally

## 2023-06-27 v0.4.8

- Add `all` flag to `project remove`
- Add a select and date picker to `project schedule`

## 2023-06-19 v0.4.7

- Don't `reschedule` overdue recurring tasks (as it wipes out the recurring data)
- Add `auto` flag to `project remove`

## 2023-06-10 v0.4.6

- Fix out of date version text
- Add `overdue` flag to `project schedule`
- Add `due` flag to `task create`
- Add `project rename`

## 2023-06-06 v0.4.5

- Show item count in `project list`
- Add remaining task count to `task next`

## 2023-06-04 v0.4.4

- Add description flag to `task create`
- Add ability to skip tasks in `project prioritize` and `project empty`
- Improve naming of priorities in prompt
- Add `version check` to check if Tod is on the latest version

## 2023-05-31 v0.4.3

- Add `task edit`. Thank you `@BobToninho`
- Improve error when no projects found in config

## 2023-05-28 v0.4.2

- Internal refactor of priority code. Thank you, `@titoOdUA`
- Clarify scheduled flag in help
- Add priority flag to `task create`
- Add `project import`
- Remove support for legacy path
- Create random config file for each test

## 2023-05-14 v0.4.1

- Speed up CI by improving caching
- Improve formatting of tasks both individually and in lists
- Return an error during complete task if there is no next task in config. Thank you, `@titoOdUA`
- Add ability to skip a task in `project process` command. Thank you, `@titoOdUA`
- Refactor to reduce the use of clone by passing references

## 2023-05-06 v0.4.0

- Break the whole API and move over to GitHub CLI inspired commands i.e. `task create` instead of `-tp`. This opens the path to adding many new features and eases the maintenance burden

## 2023-04-15 v0.3.11

- Update tod.gif to demonstrate current feel of app
- Check for `TODO` and `dbg!` on CI
- Allow completing tasks as one of the options when dating them

## 2023-04-12 v0.3.10

- Include overdue tasks in date tasks functionality

## 2023-04-09 v0.3.9

- Added ability to date tasks without due dates with `-d`

## 2023-04-08 v0.3.8

- Removed some `dbg!` statements accidentally left in the code

## 2023-04-08 v0.3.7

- Prompt for section when moving an item to a project with sections

## 2023-04-03 v0.3.6

- Re-releasing due to an HTTP error on publish that could not be redone with the same version number

## 2023-04-03 v0.3.5

- Add the flag `x` to get the next item one at a time with an interactive prompt
- Add additional test coverage

## 2023-04-01 v0.3.4

- Improve input prompts with the `inquire` library
- Update dependencies

## 2023-03-21 v0.3.3

- Add the ability to disable spinners in the config file with `"spinners": false`

## 2023-03-19 v0.3.2

- Resolve new Clippy warnings
- Use a constant for the time in tests
- Add spinners to API requests

## 2022-10-18 v0.3.1

- Fix for deprecation of token passed in the request body (using Bearer Token header now), previous versions of Tod do not work now
- Add a message when the config file is created
- Differentiate between no flags and wrong flags when unrecognized input

## 2022-10-18 v0.3.0

- `Todoist` removed their v8 Sync API, this update switches to v9

## 2022-10-02 v0.2.15

- Update clap to new major version (it had breaking changes)
- Add a shell script for manual testing that hits the `Todoist` API

## 2022-07-16 v0.2.14

- Fix a bug where the config file is moved, but the path inside the config file is not altered
- Update dependencies

## 2022-07-06 v0.2.13

- UPDATE: DO NOT USE THIS VERSION, use 0.2.14 instead, as the config change introduces a bug
- Move config from `~/.tod.cfg` to `$XDG_CONFIG_HOME/tod.cfg` (i.e. `~/.config/tod.cfg`). Thank you `@hyblm`!
- If the config is in the old location, it will be moved to the new one
- Update dependencies

## 2022-04-30 v0.2.12

- List all tasks by using -p (projects) without other flags

## 2022-02-27 v0.2.11

- Add support for custom configuration path
- Update dependencies
- Add Dependabot
- Add timezone to test cases
- Update clap and alter deprecated code

## 2022-01-01 v0.2.10

- Update to 2021 Edition
- Update dependencies

## 2021-11-06 v0.2.9

- Get the timezone from the user and use that for formatting. (Previously defaulted to Pacific)
- Use an Item fixture for test setup
- Update dependencies

## 2021-10-23 v0.2.8

- Fix timezone for formatted date times

## 2021-10-18 v0.2.7

- Default to Pacific Timezone instead of UTC when no timezone in response (sorry for people not on the Wet Coast, I will be adding time zones to the config shortly)
- Improve publish instructions
- Update dependencies

## 2021-10-15 v0.2.6

- Update dependencies
- Use exit code 1 when an error occurs, otherwise 0
- Use the timezone specified by the `Todoist` response

## 2021-10-13 v0.2.5

- Update dependencies
- Adding releases
- Added MIT license
- Git ignore binaries
- Added some philosophical ramblings to README

## 2021-10-03 v0.2.4

- Hotfix for moving items to different projects

## 2021-10-02 v0.2.3

- Check for the latest version once per day and prompt to update with `cargo install tod`
- Update dependencies
- Only show the time when a Date Time is today
- Code cleanup around error handling
- Code cleanup around handling

## 2021-09-25 v0.2.2

- Sort projects alphabetically when listing
- Additional test coverage
- Publish checklist
- Prioritize overdue items when fetching the next item
- Add an ASCII icon ↻ for when an item is recurring

## 2021-09-20 v0.2.1

- Add `--scheduled` feature
- Refactor of codebase
- Add pretty assertions and `Mockito` to dev dependencies
- Can now use natural language for creating tasks outside of inbox (sends a task to inbox and then moves it to the other project)

## 2021-09-14 v0.2.0

- Breaking changes to command line arguments. Switched over to Clap crate for parsing arguments which will help handle additional features
- Add `--complete` feature
- Set priority of no date to 80
- Set the date as Today when it is... today
- Add `--sort` feature
- Add `--prioritize` feature
- Don't show items that are not today in the `--next` command
- Update dependencies

## 2021-09-07 v0.1.2

- Color the item content based on the priority assigned
- Print the item description below the content if exists
- Print the date below the description if exists
- Update dependencies

## 2021-09-06 v0.1.1

- Breaking changes to `.tod.cfg` (added new keys)
- Fetches the next item from your to-do list based on
  - Date
  - Time
  - Priority
  - If it is recurring
  
  Items are ranked by points and the first is returned:

  ```monospace
    Date is today with no time: 100
    Date is today with time in next or last 15 min: 200
    Item has no priority: 2
    Priority 1: 1
    Priority 2: 3
    Priority 3: 4
    Not recurring: 50
  ```

- Saves the config file rather than deleting and recreating
- Add tarpaulin for test coverage
- Add shell script `test.sh` for testing
- Increase test coverage

## 2020-11-11 v0.1.0

- The first commit!
- Add and remove projects
- Create a task in either inbox or a project
