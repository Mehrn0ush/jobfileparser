use getopts::Options;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use encoding_rs::UTF_16LE;
use encoding_rs_io::DecodeReaderBytesBuilder;
use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Debug)]
struct JobDate {
    year: u16,
    month: u16,
    weekday: Option<u16>,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
}

impl JobDate {
    fn new(data: &[u8], scheduled: bool) -> JobDate {
        let year = u16::from_le_bytes([data[0], data[1]]);
        let month = u16::from_le_bytes([data[2], data[3]]);
        let weekday = if !scheduled {
            Some(u16::from_le_bytes([data[4], data[5]]))
        } else {
            None
        };
        let day = u16::from_le_bytes([data[6], data[7]]);
        let hour = u16::from_le_bytes([data[8], data[9]]);
        let minute = u16::from_le_bytes([data[10], data[11]]);
        let second = u16::from_le_bytes([data[12], data[13]]);
        JobDate {
            year,
            month,
            weekday,
            day,
            hour,
            minute,
            second,
        }
    }

    fn format_date(&self) -> String {
        let weekdays = [
            "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday",
        ];
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        if let Some(weekday) = self.weekday {
            format!(
                "{} {} {} {:02}:{:02}:{:02} {}",
                weekdays[weekday as usize],
                months[self.month as usize - 1],
                self.day,
                self.hour,
                self.minute,
                self.second,
                self.year
            )
        } else {
            format!(
                "{} {} {:02}:{:02}:{:02} {}",
                months[self.month as usize - 1],
                self.day,
                self.hour,
                self.minute,
                self.second,
                self.year
            )
        }
    }
}

#[derive(Debug)]
struct UUID {
    uuid0: u32,
    uuid1: u16,
    uuid2: u16,
    uuid3: u16,
    uuid4: u16,
    uuid5: u16,
    uuid6: u16,
}

impl UUID {
    fn new(data: &[u8]) -> UUID {
        UUID {
            uuid0: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            uuid1: u16::from_le_bytes([data[4], data[5]]),
            uuid2: u16::from_le_bytes([data[6], data[7]]),
            uuid3: u16::from_be_bytes([data[8], data[9]]),
            uuid4: u16::from_be_bytes([data[10], data[11]]),
            uuid5: u16::from_be_bytes([data[12], data[13]]),
            uuid6: u16::from_be_bytes([data[14], data[15]]),
        }
    }

    fn format_uuid(&self) -> String {
        format!(
            "{{{:08X}-{:04X}-{:04X}-{:04X}-{:02X}{:02X}{:02X}}}",
            self.uuid0, self.uuid1, self.uuid2, self.uuid3, self.uuid4, self.uuid5, self.uuid6
        )
    }
}

#[derive(Debug)]
struct Job {
    product_info: u16,
    file_version: u16,
    uuid: UUID,
    priority: u32,
    max_run_time: i32,
    exit_code: i32,
    status: i32,
    flags: u32,
    run_date: JobDate,
    scheduled_date: JobDate,
    name: String,
    parameters: String,
    working_directory: String,
    user: String,
    comment: String,
}

impl Job {
    fn new(data: &[u8]) -> Job {
        let product_info = u16::from_le_bytes([data[0], data[1]]);
        let file_version = u16::from_le_bytes([data[2], data[3]]);
        let uuid = UUID::new(&data[4..20]);
        let priority = u32::from_le_bytes([data[32], data[33], data[34], data[35]]);
        let max_run_time = i32::from_le_bytes([data[36], data[37], data[38], data[39]]);
        let exit_code = i32::from_le_bytes([data[40], data[41], data[42], data[43]]);
        let status = i32::from_le_bytes([data[44], data[45], data[46], data[47]]);
        let flags = u32::from_le_bytes([data[48], data[49], data[50], data[51]]);
        let run_date = JobDate::new(&data[52..68], false);
        let scheduled_date = JobDate::new(&data[68..88], true);
        let name_length = u16::from_le_bytes([data[70], data[71]]);
        let name = std::str::from_utf8(&data[72..72 + name_length as usize * 2])
            .unwrap()
            .replace('\x00', "");
        let parameter_size = u16::from_le_bytes([data[72 + name_length as usize * 2], data[73 + name_length as usize * 2]]);
        let parameters = std::str::from_utf8(&data[74 + name_length as usize * 2..74 + name_length as usize * 2 + parameter_size as usize * 2])
            .unwrap()
            .replace('\x00', "");
        let working_directory_size = u16::from_le_bytes([data[74 + name_length as usize * 2 + parameter_size as usize * 2], data[75 + name_length as usize * 2 + parameter_size as usize * 2]]);
        let working_directory = std::str::from_utf8(&data[76 + name_length as usize * 2 + parameter_size as usize * 2..76 + name_length as usize * 2 + parameter_size as usize * 2 + working_directory_size as usize * 2])
            .unwrap()
            .replace('\x00', "");
        let user_size = u16::from_le_bytes([data[76 + name_length as usize * 2 + parameter_size as usize * 2 + working_directory_size as usize * 2], data[77 + name_length as usize * 2 + parameter_size as usize * 2 + working_directory_size as usize * 2]]);
        let user = std::str::from_utf8(&data[78 + name_length as usize * 2 + parameter_size as usize * 2 + working_directory_size as usize * 2..78 + name_length as usize * 2 + parameter_size as usize * 2 + working_directory_size as usize * 2 + user_size as usize * 2])
            .unwrap()
            .replace('\x00', "");
        let comment_size = u16::from_le_bytes([data[78 + name_length as usize * 2 + parameter_size as usize * 2 + working_directory_size as usize * 2 + user_size as usize * 2], data[79 + name_length as usize * 2 + parameter_size as usize * 2 + working_directory_size as usize * 2 + user_size as usize * 2]]);
        let comment = std::str::from_utf8(&data[80 + name_length as usize * 2 + parameter_size as usize * 2 + working_directory_size as usize * 2 + user_size as usize * 2..80 + name_length as usize * 2 + parameter_size as usize * 2 + working_directory_size as usize * 2 + user_size as usize * 2 + comment_size as usize * 2])
            .unwrap()
            .replace('\x00', "");

        Job {
            product_info,
            file_version,
            uuid,
            priority,
            max_run_time,
            exit_code,
            status,
            flags,
            run_date,
            scheduled_date,
            name,
            parameters,
            working_directory,
            user,
            comment,
        }
    }

    fn format_job(&self) -> String {
        let products: HashMap<u16, &str> = vec![
            (0x400, "Windows NT 4.0"),
            (0x500, "Windows 2000"),
            (0x501, "Windows XP"),
            (0x600, "Windows Vista"),
            (0x601, "Windows 7"),
            (0x602, "Windows 8"),
            (0x603, "Windows 8.1"),
            (0xa00, "Windows 10"),
        ]
        .into_iter()
        .collect();

        let task_status: HashMap<i32, &str> = vec![
            (0x41300, "Task is ready to run"),
            (0x41301, "Task is running"),
            (0x41302, "Task is disabled"),
            (0x41303, "Task has not run"),
            (0x41304, "No more scheduled runs"),
            (0x41305, "Properties not set"),
            (0x41306, "Last run terminated by user"),
            (0x41307, "No triggers/triggers disabled"),
            (0x41308, "Triggers do not have set run times"),
        ]
        .into_iter()
        .collect();

        let flags: HashMap<u32, &str> = vec![
            (0x1, "TASK_APPLICATION_NAME"),
            (0x200000, "TASK_FLAG_RUN_ONLY_IF_LOGGED_ON"),
            (0x100000, "TASK_FLAG_SYSTEM_REQUIRED"),
            (0x80000, "TASK_FLAG_RESTART_ON_IDLE_RESUME"),
            (0x40000, "TASK_FLAG_RUN_IF_CONNECTED_TO_INTERNET"),
            (0x20000, "TASK_FLAG_HIDDEN"),
            (0x10000, "TASK_FLAG_RUN_ONLY_IF_DOCKED"),
            (0x80000000, "TASK_FLAG_KILL_IF_GOING_ON_BATTERIES"),
            (0x40000000, "TASK_FLAG_DONT_START_IF_ON_BATTERIES"),
            (0x20000000, "TASK_FLAG_KILL_ON_IDLE_END"),
            (0x10000000, "TASK_FLAG_START_ONLY_IF_IDLE"),
            (0x4000000, "TASK_FLAG_DISABLED"),
            (0x2000000, "TASK_FLAG_DELETE_WHEN_DONE"),
            (0x1000000, "TASK_FLAG_INTERACTIVE"),
        ]
        .into_iter()
        .collect();

        let priorities: HashMap<u32, &str> = vec![
            (0x20000000, "NORMAL_PRIORITY_CLASS"),
            (0x40000000, "IDLE_PRIORITY_CLASS"),
            (0x80000000, "HIGH_PRIORITY_CLASS"),
            (0x100000, "REALTIME_PRIORITY_CLASS"),
        ]
        .into_iter()
        .collect();

        let mut result = String::new();

        result.push_str(&format!(
            "Product Info: {}\n",
            products.get(&self.product_info).unwrap_or(&"Unknown Version")
        ));
        result.push_str(&format!("File Version: {}\n", self.file_version));
        result.push_str(&format!("UUID: {}\n", self.uuid.format_uuid()));

        let mut priority_list = String::new();
        for (key, value) in &priorities {
            if self.priority & key == *key {
                priority_list.push_str(value);
                priority_list.push_str(", ");
            }
        }
        if !priority_list.is_empty() {
            result.push_str(&format!(
                "Priorities: {}\n",
                priority_list.trim_end_matches(", ")
            ));
        }

        let hours = self.max_run_time / 3600000;
        let ms = self.max_run_time % 3600000;
        let minutes = ms / 60000;
        let ms = ms % 60000;
        let seconds = ms / 1000;
        let ms = ms % 1000;
        result.push_str(&format!(
            "Maximum Run Time: {:02}:{:02}:{:02}.{} (HH:MM:SS.MS)\n",
            hours, minutes, seconds, ms
        ));
        result.push_str(&format!("Exit Code: {}\n", self.exit_code));
        result.push_str(&format!(
            "Status: {}\n",
            task_status.get(&self.status).unwrap_or(&"Unknown Status")
        ));

        let mut flag_list = String::new();
        for (key, value) in &flags {
            if self.flags & key == *key {
                flag_list.push_str(value);
                flag_list.push_str(", ");
            }
        }
        result.push_str(&format!("Flags: {}\n", flag_list.trim_end_matches(", ")));
        result.push_str(&format!("Date Run: {}\n", self.run_date.format_date()));
        result.push_str(&format!("Scheduled Date: {}\n", self.scheduled_date.format_date()));
        result.push_str(&format!("Application: {}\n", self.name));
        result.push_str(&format!("Parameters: {}\n", self.parameters));
        result.push_str(&format!("Working Directory: {}\n", self.working_directory));
        result.push_str(&format!("User: {}\n", self.user));
        result.push_str(&format!("Comment: {}\n", self.comment));

        result
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename = "Task")]
struct Task {
    #[serde(rename = "RegistrationInfo")]
    registration_info: RegistrationInfo,
    #[serde(rename = "Triggers")]
    triggers: Triggers,
    #[serde(rename = "Settings")]
    settings: Settings,
    #[serde(rename = "Actions")]
    actions: Actions,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "RegistrationInfo")]
struct RegistrationInfo {
    #[serde(rename = "Author")]
    author: Option<String>,
    #[serde(rename = "Date")]
    date: Option<String>,
    #[serde(rename = "Description")]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "Triggers")]
struct Triggers {
    #[serde(rename = "CalendarTrigger", default)]
    calendar_trigger: Option<CalendarTrigger>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "CalendarTrigger")]
struct CalendarTrigger {
    #[serde(rename = "StartBoundary")]
    start_boundary: String,
    #[serde(rename = "EndBoundary")]
    end_boundary: Option<String>,
    #[serde(rename = "Enabled")]
    enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "Settings")]
struct Settings {
    #[serde(rename = "Enabled")]
    enabled: Option<bool>,
    #[serde(rename = "AllowStartIfOnBatteries")]
    allow_start_if_on_batteries: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "Actions")]
struct Actions {
    #[serde(rename = "Exec")]
    exec: Option<Exec>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "Exec")]
struct Exec {
    #[serde(rename = "Command")]
    command: String,
    #[serde(rename = "Arguments")]
    arguments: Option<String>,
}

fn display_xml_job_info(task: &Task) {
    println!("Author: {:?}", task.registration_info.author);
    println!("Date: {:?}", task.registration_info.date);
    println!("Description: {:?}", task.registration_info.description);

    if let Some(trigger) = &task.triggers.calendar_trigger {
        println!("StartBoundary: {}", trigger.start_boundary);
        println!("EndBoundary: {:?}", trigger.end_boundary);
        println!("Enabled: {:?}", trigger.enabled);
    }

    println!("Settings:");
    println!("  Enabled: {:?}", task.settings.enabled);
    println!("  AllowStartIfOnBatteries: {:?}", task.settings.allow_start_if_on_batteries);

    if let Some(exec) = &task.actions.exec {
        println!("Command: {}", exec.command);
        println!("Arguments: {:?}", exec.arguments);
    }
}

fn decode_utf16_file<P: AsRef<Path>>(path: P) -> Result<Task, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let transcoded = DecodeReaderBytesBuilder::new()
        .encoding(Some(UTF_16LE))
        .build(file);
    let mut buffer = String::new();
    let mut reader = transcoded.take(1 << 16);
    reader.read_to_string(&mut buffer)?;
    let task: Task = from_str(&buffer)?;
    Ok(task)
}

fn usage() {
    println!("jobparser.rs:");
    println!(" -f <job>");
    println!(" -d <directory of job files>");
}

fn parse_file(file_path: &str) {
    let path = Path::new(file_path);

    if path.extension().and_then(|s| s.to_str()) == Some("xml") {
        // Try to parse as an XML job file
        match decode_utf16_file(&path) {
            Ok(task) => display_xml_job_info(&task),
            Err(e) => eprintln!("Unable to process file {}: {}", file_path, e),
        }
    } else {
        // Try to parse as a binary job file
        let mut file = File::open(&path).expect("Unable to open file");
        let mut data = Vec::new();
        file.read_to_end(&mut data).expect("Unable to read file");
        let job = Job::new(&data);
        println!("************************************************************************");
        println!("File: {}", path.display());
        println!("{}", job.format_job());
        println!("************************************************************************");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("f", "file", "set job file", "FILE");
    opts.optopt("d", "dir", "set directory of job files", "DIR");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("Error: {}", f);
            usage();
            return;
        }
    };

    if matches.opt_present("h") {
        usage();
        return;
    }

    let file_path = matches.opt_str("f");
    let dir_path = matches.opt_str("d");

    if file_path.is_none() && dir_path.is_none() {
        usage();
        return;
    }

    if let Some(dir) = dir_path {
        if Path::new(&dir).is_dir() {
            for entry in fs::read_dir(dir).expect("Unable to read directory") {
                let entry = entry.expect("Unable to get entry");
                let path = entry.path();
                if path.is_file() && (path.extension().and_then(|s| s.to_str()) == Some("job") || path.extension().and_then(|s| s.to_str()) == Some("xml")) {
                    parse_file(path.to_str().unwrap());
                }
            }
        }
    } else if let Some(file_path) = file_path {
        parse_file(&file_path);
    }
}
