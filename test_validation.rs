fn main() {
    let input_text = "keithynlewis-jpg, adelaluxzepeda-w, ja";
    
    let usernames: Vec<String> = input_text.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .filter(|s| s.len() <= 39) // GitHub username max length
        .filter(|s| s.chars().all(|c| c.is_alphanumeric() || c == '-'))
        .map(|s| s.to_string())
        .collect();
    
    println!("Input: {}", input_text);
    println!("Parsed usernames: {:?}", usernames);
    
    for username in &usernames {
        println!("Username: '{}', Length: {}, Valid chars: {}", 
                 username, 
                 username.len(),
                 username.chars().all(|c| c.is_alphanumeric() || c == '-'));
    }
}
