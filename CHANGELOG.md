# Changelog

## Unreleased

- Prompt for section when moving an item to a project with sections

## 2023-04-03 v0.3.6

- Re-releasing due to an HTTP error on publish that could not be redone with the same version number.

## 2023-04-03 v0.3.5

- Add the flag `x` to get the next item one at a time with an interactive prompt
- Add additional test coverage

## 2023-04-01 v0.3.4

- Improve input prompts with the inquire library
- Update dependencies

## 2023-03-21 v0.3.3

- Add the ability to disable spinners in the config file with `"spinners": false`

## 2023-03-19 v0.3.2

- Resolve new Clippy warnings
- Use a constant for the time in tests
- Add spinners to API requests

## 2022-10-18 v0.3.1

- Fix for deprecation of token passed in the request body (using Bearer Token header now), previous versions of Tod do not work anymore.
- Add a message when the config file is created
- Differentiate between no flags and wrong flags when unrecognized input

## 2022-10-18 v0.3.0

- Todoist removed their v8 Sync API, this update switches to v9

## 2022-10-02 v0.2.15

- Update clap to new major version (it had breaking changes)
- Add a shell script for manual testing that hits the Todoist API

## 2022-07-16 v0.2.14

- Fix a bug where the config file is moved but the path inside the config file is not altered
- Update dependencies

## 2022-07-06 v0.2.13

- UPDATE: DO NOT USE THIS VERSION, use 0.2.14 instead, as the config change introduces a bug
- Move config from `~/.tod.cfg` to `$XDG_CONFIG_HOME/tod.cfg` (i.e. `~/.config/tod.cfg`). Thank you @hyblm!
- If the config is in the old location, it will be moved to the new one
- Update dependencies

## 2022-04-30 v0.2.12

- List all tasks by using -p (projects) without other flags

## 2022-02-27 v0.2.11

- Add support for custom configuration path
- Update dependencies
- Add Dependabot
- Add timezone to test cases
- Update clap and resolve deprecations

## 2022-01-01 v0.2.10

- Update to 2021 Edition
- Update dependencies

## 2021-11-06 v0.2.9

- Get the timezone from the user and use that for formatting. (Previously defaulted to Pacific)
- Use an Item fixture for test setup
- Update dependencies

## 2021-10-23 v0.2.8

- Fix timezone for formatted datetimes

## 2021-10-18 v0.2.7

- Default to Pacific Timezone instead of UTC when no timezone in response (sorry for people not on the Wet Coast, I will be adding time zones to the config shortly)
- Improve publish instructions
- Update dependencies

## 2021-10-15 v0.2.6

- Update dependencies
- Use exit code 1 when an error occurs, otherwise 0
- Use the timezone specified by the Todoist response

## 2021-10-13 v0.2.5

- Update dependencies
- Adding releases
- Added MIT license
- GitIgnore binaries
- Added some philosophical ramblings to README

## 2021-10-03 v0.2.4

- Hotfix for moving items to different projects

## 2021-10-02 v0.2.3

- Check for the latest version once per day and prompt to update with `cargo install tod`
- Update dependencies
- Only show the time when a DateTime is today
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
- Add pretty assertions and Mockito to dev dependencies
- Can now use natural language for creating tasks outside of inbox (sends a task to inbox and then moves it to the other project)

## 2021-09-14 v0.2.0

- Breaking changes to command line arguments. Switched over to Clap crate for parsing arguments which will help handle additional features.
- Add `--complete` feature
- Set priority of no date to 80
- Set the date as Today when it is... today.
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
- Fetches the next item from your todo list based on
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
