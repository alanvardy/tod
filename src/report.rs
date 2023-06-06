use std::fmt::Display;

use crate::{
    config::Config,
    items::{self, Item},
    projects,
};

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
    project_name: String,
    items: Vec<Item>,
    config: Config,
    report_type: CommonReports,
}

impl Report {
    pub fn new(config: Config, project: &str, report_type: CommonReports) -> Self {
        match report_type {
            CommonReports::DoneYesterday => Report::form_done_yesterday_report(config, project),
            CommonReports::DoneToday => Report::form_done_today_report(config, project),
            CommonReports::DueToday => Report::form_due_today_report(config, project),
        }
    }

    fn form_done_yesterday_report(config: Config, project: &str) -> Self {
        let project_id = projects::project_id(&config, project).expect("Failed to get project ID");
        let items = crate::todoist::completed_items_for_project(&config, &project_id)
            .expect("Failed to get completed items for a project");
        let items: Vec<Item> = items
            .into_iter()
            .filter(|item| {
                item.get_completed_at(&config).date_naive()
                    == crate::time::now(&config).date_naive() - chrono::Duration::days(1)
            })
            .collect();
        Report {
            config,
            project_name: project.to_string(),
            items,
            report_type: CommonReports::DoneYesterday,
        }
    }

    fn form_done_today_report(config: Config, project: &str) -> Self {
        let project_id = projects::project_id(&config, project).expect("Failed to get project ID");
        let items = crate::todoist::completed_items_for_project(&config, &project_id)
            .expect("Failed to get completed items for a project");
        let items: Vec<Item> = items
            .into_iter()
            .filter(|item| {
                item.get_completed_at(&config).date_naive()
                    == crate::time::now(&config).date_naive()
            })
            .collect();
        Report {
            config,
            project_name: project.to_string(),
            items,
            report_type: CommonReports::DoneToday,
        }
    }

    fn form_due_today_report(config: Config, project: &str) -> Report {
        let project_id = projects::project_id(&config, project).expect("Failed to get project ID");
        let items = crate::todoist::items_for_project(&config, &project_id)
            .expect("Failed to get items for a project");
        let items: Vec<Item> = items
            .into_iter()
            .filter(|item| item.is_today(&config))
            .collect();
        Report {
            config,
            project_name: project.to_string(),
            items,
            report_type: CommonReports::DoneToday,
        }
    }

    pub fn print(&self) -> Result<String, String> {
        let mut buffer = String::new();
        buffer.push_str(&projects::green_string(&format!(
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
