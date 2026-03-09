use crate::task::{TASK_COSTS, TASK_PRIORITIES, TASK_TIMES};

pub struct Util;

impl Util {
    pub fn get_spaced_title(title: &str) -> String {
        format!(" {} ", title)
    }

    pub fn encode_filename(title: &str) -> String {
        // Trim whitespace from the beginning and end
        let title = title.trim();
        let mut result = String::new();

        for c in title.chars() {
            match c {
                // Keep spaces as literal spaces
                ' ' => result.push(' '),
                // Percent sign encodes to %25
                '%' => result.push_str("%25"),
                // Invalid filename characters encode to %XX hex
                '/' => result.push_str("%2F"),
                '\\' => result.push_str("%5C"),
                '<' => result.push_str("%3C"),
                '>' => result.push_str("%3E"),
                ':' => result.push_str("%3A"),
                '"' => result.push_str("%22"),
                '|' => result.push_str("%7C"),
                '?' => result.push_str("%3F"),
                '*' => result.push_str("%2A"),
                // Keep other ASCII alphanumeric and safe chars (including spaces now)
                c if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' => {
                    result.push(c)
                }
                // Other special chars also encode to %XX
                c => {
                    // Use percent-encoding for any other non-ASCII or special chars
                    result.push('%');
                    result.push_str(&format!("{:02X}", c as u32));
                }
            }
        }

        // Trim leading/trailing dots which can be problematic on some filesystems
        result = result
            .trim_start_matches('.')
            .trim_end_matches('.')
            .to_string();

        // If empty after trimming, use a default
        if result.is_empty() {
            result = "task".to_string();
        }

        result
    }

    pub fn decode_filename(filename: &str) -> String {
        let mut result = String::new();
        let mut chars = filename.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '%' {
                // Try to read the next two hex digits
                let hex1 = chars.next();
                let hex2 = chars.next();

                if let (Some(h1), Some(h2)) = (hex1, hex2) {
                    let hex_str = format!("{}{}", h1, h2);
                    if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                        result.push(byte as char);
                    } else {
                        // Invalid hex sequence, keep the % and chars as-is
                        result.push('%');
                        result.push(h1);
                        result.push(h2);
                    }
                } else {
                    // Truncated % sequence, keep as-is
                    result.push('%');
                    if let Some(h1) = hex1 {
                        result.push(h1);
                    }
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    pub fn get_priority_indicator(value: u8) -> String {
        // Priority value is in ascending order
        // but in the visualization the order is reversed to be more intuitive
        // priority: 1 => !!!
        // priority: 2 => !!
        // priority: 3 => !
        let priority_value = if value == 0 {
            0
        } else {
            TASK_PRIORITIES
                .into_iter()
                .rev()
                .position(|t| t == value)
                .unwrap_or(0)
        };

        // Get the indicator chars and pad OUTSIDE the brackets for alignment
        // [!!!] = 5 chars, [!!] needs 1 space, [!] needs 2 spaces
        let indicator: String = "!!!".chars().take((priority_value).into()).collect();
        let spaces = "  ".chars().take(3 - priority_value).collect::<String>();
        format!("[{}]{}", indicator, spaces)
    }

    pub fn get_cost_indicator(value: u8) -> String {
        // Cost value is in ascending order
        // cost: 1 => $$$
        // cost: 2 => $$
        // cost: 3 => $
        let cost_value = if value == 0 {
            0
        } else {
            TASK_COSTS
                .into_iter()
                .rev()
                .position(|t| t == value)
                .unwrap_or(0)
        };

        "$$$".chars().take((cost_value).into()).collect()
    }

    pub fn get_time_indicator(value: u8) -> String {
        // Time value is in ascending order
        // time: 1 => ⏲⏲⏲
        // time: 2 => ⏲⏲
        // time: 3 => ⏲
        let time_value = if value == 0 {
            0
        } else {
            TASK_TIMES
                .into_iter()
                .rev()
                .position(|t| t == value)
                .unwrap_or(0)
        };

        "⏲⏲⏲".chars().take((time_value).into()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_and_decode_filename() {
        // Spaces are kept as literal spaces, only special chars are percent-encoded
        let test_cases = vec![
            ("Hello World", "Hello World"),
            ("Review & Test", "Review %26 Test"),
            ("Task: Important!", "Task%3A Important%21"),
            ("Bug #123", "Bug %23123"),
            ("100% Complete", "100%25 Complete"),
            ("A/B Testing", "A%2FB Testing"),
            ("Special <chars>", "Special %3Cchars%3E"),
            ("no_changes", "no_changes"),
            ("Space   here", "Space   here"),
        ];

        for (original, expected_encoded) in test_cases {
            let encoded = Util::encode_filename(original);
            assert_eq!(
                encoded, expected_encoded,
                "Failed encoding for: {}",
                original
            );

            let decoded = Util::decode_filename(&encoded);
            assert_eq!(decoded, original, "Failed decoding for: {}", encoded);
        }
    }

    #[test]
    fn test_encode_empty() {
        // Empty string returns "task"
        assert_eq!(Util::encode_filename(""), "task");
        // Only dots returns "task"
        assert_eq!(Util::encode_filename("..."), "task");
        // Whitespace-only strings are trimmed and return "task"
        assert_eq!(Util::encode_filename("   "), "task");
        assert_eq!(Util::encode_filename("  \t\n  "), "task");
    }
}
