#[cfg(test)]
mod tests {
    use rand::Rng;
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    #[test]
    fn simple_semaphore() {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let semaphore = Arc::new(Semaphore::new(4)); // max 4 tasks

                let mut handles = vec![];

                for i in 0..10 {
                    // assuming we have 10 tasks
                    let permit = semaphore.clone().acquire_owned().await.unwrap();
                    let random_delay: u64 = rand::thread_rng().gen_range(10..=580) + 500;

                    handles.push(tokio::spawn(async move {
                        println!("Task {} started", i);
                        tokio::time::sleep(tokio::time::Duration::from_millis(random_delay)).await; // simulate task work
                        println!("Task {} ended : Delay : {}", i, random_delay);
                        drop(permit);
                    }));
                }

                // Wait for all tasks to complete
                for handle in handles {
                    handle.await.unwrap();
                }
            })
    }
}
