# Changelog

## Unreleased

## 2021-09-07 0.1.2
- Color the item content based on the priority assigned
- Print the item description below the content if exists
- Print the date below the description if exists

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
