pub fn format_progress(current: usize, total: usize, prefix: &str, unit: &str) -> String {
    let width = 20;
    let percentage = if total > 0 { (current as f64 / total as f64 * 100.0) as usize } else { 100 };
    let filled = if total > 0 { (current * width) / total } else { width };
    let empty = width.saturating_sub(filled);
    
    let bar = format!("{}{}", "█".repeat(filled), " ".repeat(empty));
    format!("{} [{}] {}% ({}/{} {})", prefix, bar, percentage, current, total, unit)
}

pub fn print_progress(current: usize, total: usize, prefix: &str, unit: &str) {
    use std::io::{self, Write};
    let s = format_progress(current, total, prefix, unit);
    print!("\r{}", s);
    if current == total || total == 0 {
        println!();
    }
    let _ = io::stdout().flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_progress() {
        let out = format_progress(5, 10, "Indexing", "TUs");
        assert_eq!(out, "Indexing [██████████          ] 50% (5/10 TUs)");
        
        let out2 = format_progress(10, 10, "Indexing", "TUs");
        assert_eq!(out2, "Indexing [████████████████████] 100% (10/10 TUs)");
        
        let out3 = format_progress(0, 10, "Indexing", "TUs");
        assert_eq!(out3, "Indexing [                    ] 0% (0/10 TUs)");
    }
}
