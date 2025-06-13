use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_duration: Duration,
    pub chunk_count: usize,
    pub average_chunk_time: Duration,
    pub fastest_chunk: Duration,
    pub slowest_chunk: Duration,
    pub api_calls: usize,
    pub failed_api_calls: usize,
    pub total_characters_processed: usize,
    pub characters_per_second: f64,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PerformanceTracker {
    start_time: Instant,
    chunk_times: Vec<Duration>,
    api_call_times: Vec<Duration>,
    failed_calls: usize,
    total_characters: usize,
    metrics: HashMap<String, Duration>,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            chunk_times: Vec::new(),
            api_call_times: Vec::new(),
            failed_calls: 0,
            total_characters: 0,
            metrics: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn start_chunk(&self) -> Instant {
        Instant::now()
    }

    #[allow(dead_code)]
    pub fn end_chunk(&mut self, start_time: Instant, characters: usize) {
        let duration = start_time.elapsed();
        self.chunk_times.push(duration);
        self.total_characters += characters;
    }

    #[allow(dead_code)]
    pub fn start_api_call(&self) -> Instant {
        Instant::now()
    }

    #[allow(dead_code)]
    pub fn end_api_call(&mut self, start_time: Instant, success: bool) {
        let duration = start_time.elapsed();
        self.api_call_times.push(duration);
        if !success {
            self.failed_calls += 1;
        }
    }

    #[allow(dead_code)]
    pub fn record_metric(&mut self, name: &str, duration: Duration) {
        self.metrics.insert(name.to_string(), duration);
    }

    #[allow(dead_code)]
    pub fn get_metrics(&self) -> PerformanceMetrics {
        let total_duration = self.start_time.elapsed();
        let chunk_count = self.chunk_times.len();
        
        let average_chunk_time = if chunk_count > 0 {
            self.chunk_times.iter().sum::<Duration>() / chunk_count as u32
        } else {
            Duration::ZERO
        };

        let fastest_chunk = self.chunk_times.iter().min().copied().unwrap_or(Duration::ZERO);
        let slowest_chunk = self.chunk_times.iter().max().copied().unwrap_or(Duration::ZERO);

        let characters_per_second = if total_duration.as_secs_f64() > 0.0 {
            self.total_characters as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        PerformanceMetrics {
            total_duration,
            chunk_count,
            average_chunk_time,
            fastest_chunk,
            slowest_chunk,
            api_calls: self.api_call_times.len(),
            failed_api_calls: self.failed_calls,
            total_characters_processed: self.total_characters,
            characters_per_second,
        }
    }

    #[allow(dead_code)]
    pub fn print_summary(&self) {
        let metrics = self.get_metrics();
        println!("📊 Performance Summary:");
        println!("   Total duration: {:.2}s", metrics.total_duration.as_secs_f64());
        println!("   Chunks processed: {}", metrics.chunk_count);
        println!("   Average chunk time: {:.2}s", metrics.average_chunk_time.as_secs_f64());
        println!("   Fastest chunk: {:.2}s", metrics.fastest_chunk.as_secs_f64());
        println!("   Slowest chunk: {:.2}s", metrics.slowest_chunk.as_secs_f64());
        println!("   API calls: {} ({} failed)", metrics.api_calls, metrics.failed_api_calls);
        println!("   Characters processed: {}", metrics.total_characters_processed);
        println!("   Processing speed: {:.1} chars/sec", metrics.characters_per_second);
        
        if !self.metrics.is_empty() {
            println!("   Custom metrics:");
            for (name, duration) in &self.metrics {
                println!("     {}: {:.2}s", name, duration.as_secs_f64());
            }
        }
    }
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_performance_tracker() {
        let mut tracker = PerformanceTracker::new();
        
        // Simulate chunk processing
        let start = tracker.start_chunk();
        thread::sleep(Duration::from_millis(10));
        tracker.end_chunk(start, 1000);
        
        // Simulate API call
        let start = tracker.start_api_call();
        thread::sleep(Duration::from_millis(5));
        tracker.end_api_call(start, true);
        
        let metrics = tracker.get_metrics();
        assert_eq!(metrics.chunk_count, 1);
        assert_eq!(metrics.api_calls, 1);
        assert_eq!(metrics.failed_api_calls, 0);
        assert_eq!(metrics.total_characters_processed, 1000);
        assert!(metrics.total_duration.as_millis() >= 15);
    }

    #[test]
    fn test_failed_api_call() {
        let mut tracker = PerformanceTracker::new();
        
        let start = tracker.start_api_call();
        tracker.end_api_call(start, false);
        
        let metrics = tracker.get_metrics();
        assert_eq!(metrics.failed_api_calls, 1);
    }

    #[test]
    fn test_custom_metrics() {
        let mut tracker = PerformanceTracker::new();
        tracker.record_metric("test_operation", Duration::from_millis(100));
        
        assert!(tracker.metrics.contains_key("test_operation"));
        assert_eq!(tracker.metrics["test_operation"], Duration::from_millis(100));
    }
}
