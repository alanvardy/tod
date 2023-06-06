use std::fmt::Display;

use crate::{
    color,
    config::Config,
    items::{self, Item},
    projects,
};

#[derive(PartialEq, Debug)]
pub enum CommonReports {
    DoneYesterday,
    DoneToday,
    DueToday,
}

impl Display for CommonReports {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommonReports::DoneYesterday => write!(f, "Tasks completed yesterday"),
            CommonReports::DoneToday => write!(f, "Tasks completed today"),
            CommonReports::DueToday => write!(f, "Tasks that need to be done today"),
        }
    }
}

pub struct Report {
    pub project_name: String,
    pub items: Vec<Item>,
    pub config: Config,
    pub report_type: CommonReports,
}

impl Report {
    pub fn new(config: Config, project: &str, report_type: CommonReports) -> Result<Self, String> {
        let project_id =
            projects::project_id(&config, project).map_err(|_| format!("Failed to get project"))?;
        match report_type {
            CommonReports::DoneYesterday => {
                let items = crate::todoist::completed_items_for_project(&config, &project_id)
                    .map_err(|_| format!("Failed to get completed items for the project"))?;
                Report::form_done_yesterday_report(config, project, items)
            }
            CommonReports::DoneToday => {
                let items = crate::todoist::completed_items_for_project(&config, &project_id)
                    .map_err(|_| format!("Failed to get completed items for the project"))?;
                Report::form_done_today_report(config, project, items)
            }
            CommonReports::DueToday => {
                let items = crate::todoist::items_for_project(&config, &project_id)
                    .map_err(|_| format!("Failed to get completed items for the project"))?;
                Report::form_due_today_report(config, project, items)
            }
        }
    }

    fn form_done_yesterday_report(
        config: Config,
        project: &str,
        items: Vec<Item>,
    ) -> Result<Self, String> {
        let items: Vec<Item> = items
            .into_iter()
            .filter(|item| {
                item.get_completed_at(&config)
                    .map(|completion_time| {
                        completion_time.date_naive()
                            == crate::time::now(&config).date_naive() - chrono::Duration::days(1)
                    })
                    .unwrap_or(false)
            })
            .collect();
        Ok(Report {
            config,
            project_name: project.to_string(),
            items,
            report_type: CommonReports::DoneYesterday,
        })
    }

    fn form_done_today_report(
        config: Config,
        project: &str,
        items: Vec<Item>,
    ) -> Result<Self, String> {
        let items: Vec<Item> = items
            .into_iter()
            .filter(|item| {
                item.get_completed_at(&config)
                    .map(|completion_time| {
                        completion_time.date_naive() == crate::time::now(&config).date_naive()
                    })
                    .unwrap_or(false)
            })
            .collect();
        Ok(Report {
            config,
            project_name: project.to_string(),
            items,
            report_type: CommonReports::DoneToday,
        })
    }

    fn form_due_today_report(
        config: Config,
        project: &str,
        items: Vec<Item>,
    ) -> Result<Self, String> {
        let items: Vec<Item> = items
            .into_iter()
            .filter(|item| item.is_today(&config))
            .collect();
        Ok(Report {
            config,
            project_name: project.to_string(),
            items,
            report_type: CommonReports::DueToday,
        })
    }

    pub fn print(&self) -> Result<String, String> {
        let mut buffer = String::new();
        buffer.push_str(&color::green_string(&format!(
            "{} in {} project:",
            self.report_type, self.project_name
        )));

        for item in self.items.iter() {
            buffer.push('\n');
            buffer.push_str(&item.fmt(&self.config, items::FormatType::List));
        }
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::{CommonReports, Report};
    use crate::{
        items::{DateInfo, Item},
        test::fixtures,
    };

    #[test]
    fn check_form_done_today_report() {
        let config = fixtures::config();
        let items = get_test_tasks();
        let project_name = "test1";

        let report = Report::form_done_today_report(config, project_name, items);
        assert!(&report.is_ok());

        let report = report.unwrap();
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.report_type, CommonReports::DoneToday);
        assert_eq!(report.project_name, project_name);
    }

    #[test]
    fn check_form_done_yesterday_report() {
        let config = fixtures::config();
        let items = get_test_tasks();
        let project_name = "test2";

        let report = Report::form_done_yesterday_report(config, project_name, items);
        assert!(&report.is_ok());

        let report = report.unwrap();
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.report_type, CommonReports::DoneYesterday);
        assert_eq!(report.project_name, project_name);
    }

    #[test]
    fn check_form_due_today_report() {
        let config = fixtures::config();
        let items = get_test_tasks();
        let project_name = "test3";

        let report = Report::form_due_today_report(config, project_name, items);
        assert!(&report.is_ok());

        let report = report.unwrap();
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.report_type, CommonReports::DueToday);
        assert_eq!(report.project_name, project_name);
    }

    fn get_test_tasks() -> Vec<Item> {
        vec![
            Item {
                id: String::from("222"),
                content: String::from("Test task 1"),
                checked: None,
                description: String::from(""),
                due: Some(DateInfo {
                    date: String::from("2061-11-13"),
                    is_recurring: false,
                    timezone: Some(String::from("America/Los_Angeles")),
                }),
                priority: crate::items::priority::Priority::Medium,
                is_deleted: None,
                is_completed: None,
                completed_at: Some(chrono::Utc::now()),
            },
            Item {
                id: String::from("222"),
                content: String::from("Test task 2"),
                checked: None,
                description: String::from(""),
                due: Some(DateInfo {
                    date: chrono::Utc::now().date_naive().to_string(),
                    is_recurring: false,
                    timezone: Some(String::from("America/Los_Angeles")),
                }),
                priority: crate::items::priority::Priority::Medium,
                is_deleted: None,
                is_completed: None,
                completed_at: None,
            },
            Item {
                id: String::from("222"),
                content: String::from("Test task 3"),
                checked: None,
                description: String::from(""),
                due: Some(DateInfo {
                    date: String::from("2061-11-13"),
                    is_recurring: false,
                    timezone: Some(String::from("America/Los_Angeles")),
                }),
                priority: crate::items::priority::Priority::Medium,
                is_deleted: None,
                is_completed: None,
                completed_at: Some(chrono::Utc::now() - chrono::Duration::days(1)),
            },
        ]
    }
}
