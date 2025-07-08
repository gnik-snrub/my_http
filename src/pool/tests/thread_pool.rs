use super::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn test_single_task_executes() {
    let rt = Runtime::new().unwrap();
    let pool = ThreadPool::new(1, rt.handle().clone());

    let flag = Arc::new(AtomicUsize::new(0));
    let flag_clone = flag.clone();

    pool.enqueue(move || {
        let flag = flag_clone.clone();
        async move {
            flag.store(1, Ordering::SeqCst);
        }
    });

    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(flag.load(Ordering::SeqCst), 1);
}

#[test]
fn test_multiple_tasks_execute() {
    let rt = Runtime::new().unwrap();
    let pool = ThreadPool::new(4, rt.handle().clone());

    let counter = Arc::new(AtomicUsize::new(0));
    for _ in 0..10 {
        let counter = counter.clone();
        pool.enqueue(move || {
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });
    }

    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(counter.load(Ordering::SeqCst), 10);
}

#[test]
fn test_parallel_execution_time() {
    let rt = Runtime::new().unwrap();
    let pool = ThreadPool::new(4, rt.handle().clone());

    let start = Instant::now();

    for _ in 0..4 {
        pool.enqueue(|| async {
            tokio::time::sleep(Duration::from_millis(100)).await;
        });
    }

    std::thread::sleep(Duration::from_millis(150));
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_millis(200));
}
