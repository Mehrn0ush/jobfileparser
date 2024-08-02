#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use quick_xml::de::from_str;

    #[derive(Debug, Deserialize)]
    struct Task {
        #[serde(rename = "RegistrationInfo")]
        registration_info: RegistrationInfo,
        #[serde(rename = "Triggers")]
        triggers: Option<Triggers>,
        #[serde(rename = "Settings")]
        settings: Settings,
        #[serde(rename = "Actions")]
        actions: Actions,
    }

    #[derive(Debug, Deserialize)]
    struct RegistrationInfo {
        #[serde(rename = "Author")]
        author: Option<String>,
        #[serde(rename = "Date")]
        date: Option<String>,
        #[serde(rename = "Description")]
        description: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct Triggers {
        #[serde(rename = "CalendarTrigger", default)]
        calendar_triggers: Vec<CalendarTrigger>,
    }

    #[derive(Debug, Deserialize)]
    struct CalendarTrigger {
        #[serde(rename = "StartBoundary")]
        start_boundary: String,
        #[serde(rename = "EndBoundary")]
        end_boundary: Option<String>,
        #[serde(rename = "Enabled")]
        enabled: Option<bool>,
    }

    #[derive(Debug, Deserialize)]
    struct Settings {
        #[serde(rename = "Enabled")]
        enabled: Option<bool>,
        #[serde(rename = "AllowStartIfOnBatteries")]
        allow_start_if_on_batteries: Option<bool>,
    }

    #[derive(Debug, Deserialize)]
    struct Actions {
        #[serde(rename = "Exec")]
        exec: Exec,
    }

    #[derive(Debug, Deserialize)]
    struct Exec {
        #[serde(rename = "Command")]
        command: String,
        #[serde(rename = "Arguments")]
        arguments: Option<String>,
    }

    #[test]
    fn test_parse_xml_job_file() {
        let job_xml = r#"
        <Task>
            <RegistrationInfo>
                <Author>Test Author</Author>
                <Date>2024-08-02T12:34:56</Date>
                <Description>Test Task</Description>
            </RegistrationInfo>
            <Triggers>
                <CalendarTrigger>
                    <StartBoundary>2024-08-02T14:00:00</StartBoundary>
                    <EndBoundary>2024-08-02T15:00:00</EndBoundary>
                    <Enabled>true</Enabled>
                </CalendarTrigger>
            </Triggers>
            <Settings>
                <Enabled>true</Enabled>
                <AllowStartIfOnBatteries>true</AllowStartIfOnBatteries>
            </Settings>
            <Actions>
                <Exec>
                    <Command>notepad.exe</Command>
                    <Arguments>/A</Arguments>
                </Exec>
            </Actions>
        </Task>
        "#;

        let task: Task = from_str(job_xml).unwrap();
        assert_eq!(task.registration_info.author.unwrap(), "Test Author");
        assert_eq!(task.registration_info.date.unwrap(), "2024-08-02T12:34:56");
        assert_eq!(task.registration_info.description.unwrap(), "Test Task");
        assert!(task.triggers.is_some());
        assert_eq!(task.triggers.unwrap().calendar_triggers[0].start_boundary, "2024-08-02T14:00:00");
        assert_eq!(task.actions.exec.command, "notepad.exe");
        assert_eq!(task.actions.exec.arguments.unwrap(), "/A");
    }
}
