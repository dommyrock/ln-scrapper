use headless_chrome::Browser;
use rand::Rng;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use tokio::task::{JoinError, JoinSet};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a semaphore maximum of 4 concurrent tasks
    let semaphore = Arc::new(Semaphore::new(4));
    let mut tasks: JoinSet<Result<(), JoinError>> = JoinSet::new();

    let contents = std::fs::read_to_string("jobs_Success_909_jobs.csv")?;
    let urls: Vec<&str> = contents.split(",").collect();

    //allows Us to share the browser across multiple tasks.
    let browser = Arc::new(Mutex::new(Browser::default().unwrap()));

    urls.into_iter().for_each(|url| {
        let sem_clone = Arc::clone(&semaphore);
        let url = url.to_owned();
        let browser = Arc::clone(&browser);
        let random_delay: u64 = rand::thread_rng().gen_range(50..=80) + 200;

        let task = tokio::spawn(async move {
            sleep(Duration::from_millis(random_delay)).await;

            let _permit = sem_clone.acquire().await.unwrap();

            println!("URL {}\nDellay {} ms", &url, random_delay);

            if let Ok(page) = browser.lock().unwrap().new_tab() {
                if let Ok(tab) = page.navigate_to(&url) {
                    if tab.wait_for_element(".show-more-less-html__markup").is_ok() {
                        if let Ok(element) = tab.find_element(".show-more-less-html__markup") {
                            println!("{}", &url);
                            let content = element.get_content().unwrap();
                            println!("{}", content);
                        }
                    }
                }
            }
        });

        tasks.spawn(task);
    });

    handle_task_results(tasks).await;

    Ok(())
}

async fn handle_task_results(mut tasks: JoinSet<Result<(), JoinError>>) {
    println!("Waiting for all tasks to complete ...\n");
    while let Some(res) = tasks.join_next().await {
        match res {
            Ok(Ok(_)) => {
                // The task completed successfully
            }
            Ok(Err(e)) => {
                // The task returned an error
                eprintln!("Task returned an error: {:?}", e);
            }
            Err(e) => {
                // The task was cancelled
                eprintln!("Task was cancelled: {:?}", e);
            }
        }
    }
}