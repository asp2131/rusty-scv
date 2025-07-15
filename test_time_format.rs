use chrono::{DateTime, Utc, Duration};

fn format_time_ago(datetime: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(datetime);
    
    let seconds = duration.num_seconds();
    let minutes = duration.num_minutes();
    let hours = duration.num_hours();
    let days = duration.num_days();
    
    if seconds < 60 {
        if seconds <= 1 {
            "just now".to_string()
        } else {
            format\!("{} seconds ago", seconds)
        }
    } else if minutes < 60 {
        if minutes == 1 {
            "1 minute ago".to_string()
        } else {
            format\!("{} minutes ago", minutes)
        }
    } else if hours < 24 {
        if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format\!("{} hours ago", hours)
        }
    } else if days < 7 {
        if days == 1 {
            "1 day ago".to_string()
        } else {
            format\!("{} days ago", days)
        }
    } else if days < 30 {
        let weeks = days / 7;
        if weeks == 1 {
            "1 week ago".to_string()
        } else {
            format\!("{} weeks ago", weeks)
        }
    } else if days < 365 {
        let months = days / 30;
        if months == 1 {
            "1 month ago".to_string()
        } else {
            format\!("{} months ago", months)
        }
    } else {
        let years = days / 365;
        if years == 1 {
            "1 year ago".to_string()
        } else {
            format\!("{} years ago", years)
        }
    }
}

fn main() {
    let now = Utc::now();
    
    // Test various time intervals
    println\!("Testing relative time formatting:");
    println\!("5 seconds ago: {}", format_time_ago(now - Duration::seconds(5)));
    println\!("1 minute ago: {}", format_time_ago(now - Duration::minutes(1)));
    println\!("5 minutes ago: {}", format_time_ago(now - Duration::minutes(5)));
    println\!("1 hour ago: {}", format_time_ago(now - Duration::hours(1)));
    println\!("3 hours ago: {}", format_time_ago(now - Duration::hours(3)));
    println\!("1 day ago: {}", format_time_ago(now - Duration::days(1)));
    println\!("3 days ago: {}", format_time_ago(now - Duration::days(3)));
    println\!("1 week ago: {}", format_time_ago(now - Duration::days(7)));
    println\!("2 weeks ago: {}", format_time_ago(now - Duration::days(14)));
    println\!("1 month ago: {}", format_time_ago(now - Duration::days(30)));
    println\!("3 months ago: {}", format_time_ago(now - Duration::days(90)));
    println\!("1 year ago: {}", format_time_ago(now - Duration::days(365)));
}
EOF < /dev/null