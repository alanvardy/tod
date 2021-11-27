# Changelog

## Unreleased
- Update dependencies

# 2021-11-06 0.2.9
- Get timezone from user and use that for formatting. (Previously defaulted to Pacific)
- Use an Item fixture for test setup
- Update dependencies

# 2021-10-23 0.2.8
- Fix timezone for formatted datetimes

# 2021-10-18 0.2.7
- Default to Pacific Timezone instead of UTC when no timezone in response (sorry for people not on the Wet Coast, I will be adding time zones to the config shortly)
- Improve publish instructions
- Update dependencies

# 2021-10-15 0.2.6
- Update dependencies
- Use exit code 1 when an error occurs, otherwise 0
- Use timezone specified by Todoist response

# 2021-10-13 0.2.5
- Update dependencies
- Adding releases
- Added MIT licence
- GitIgnore binaries
- Added some philosophical ramblings to README

# 2021-10-03 0.2.4
- Hotfix for moving items to different projects

# 2021-10-02 0.2.3
- Check for latest version once per day and prompt to update with `cargo install tod`
- Update dependencies
- Only show the time when a datetime is today
- Code cleanup around error handling
- Code cleanup around handling

# 2021-09-25 0.2.2
- Sort projects alphabetically when listing
- Additional test coverage
- Publish checklist
- Prioritize overdue items when fetching the next item
- Add ascii icon ↻ for when an item is recurring

# 2021-09-20 0.2.1
- Add `--scheduled` feature
- Refactor of codebase
- Add pretty assertions and mockito to dev dependencies
- Can now use natural language for creating tasks outside of inbox (sends task to inbox and then moves it to the other project)

## 2021-09-14 0.2.0
- Breaking changes to commandline arguments. Switched over to Clap crate for parsing arguments which will help handle additional features.
- Add `--complete` feature
- Set priority of no date to 80
- Set date as Today when it is... today.
- Add `--sort` feature
- Add `--prioritize` feature
- Don't show items that are not today in the `--next` command
- Update dependencies

## 2021-09-07 0.1.2
- Color the item content based on the priority assigned
- Print the item description below the content if exists
- Print the date below the description if exists
- Update dependencies

## 2021-09-06 0.1.1
- Breaking changes to `.tod.cfg` (added new keys)
- Fetches the next item from your todo list based on
  - Date
  - Time
  - Priority
  - If it is recurring
  
  Items are ranked by points and the first is returned:
  ```
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

## 2020-11-11 0.1.0
- First commit!
- Add and remove projects
- Create a task in either inbox or a project
