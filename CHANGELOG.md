# Changelog

## Unreleased

# 0.2.1
- Add `--scheduled` feature
- Refactor of codebase
- Add pretty assertions and mockito to dev dependencies
- Can now use natural language for creating tasks outside of inbox (sends task to inbox and then moves it to the other project)

## 0.2.0
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
