use chrono::{Datelike, Local, Timelike};

/// Formats the current datetime into a string.
pub fn formatted_datetime() -> String {
    let now = Local::now();
    format!(
        "{}-{:02}-{:02}-{:02}-{:02}-{:02}",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_formatted_datetime() {
        // Get the formatted datetime
        let formatted = formatted_datetime();

        // Print the formatted datetime
        println!("Formatted datetime: {}", formatted);

        // Create a Regex object
        let re = Regex::new(r"^\d{4}-\d{2}-\d{2}-\d{2}-\d{2}-\d{2}$").unwrap();

        // Check if the formatted datetime has the correct format
        assert!(re.is_match(&formatted));
    }
}
