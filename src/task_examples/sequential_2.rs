use headless_chrome::Browser;
use rand::Rng;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let semaphore = Arc::new(Semaphore::new(4));

    let contents = std::fs::read_to_string("jobs_Success_909_jobs.csv")?;
    let urls: Vec<&str> = contents.split(",").collect();

    let browser = Arc::new(Mutex::new(Browser::default().unwrap()));

    for url in urls {
        let sem_clone = Arc::clone(&semaphore);
        let url = url.to_owned();
        let browser = Arc::clone(&browser);
        let random_delay: u64 = rand::thread_rng().gen_range(50..=80) + 200;

        let task = tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();
            sleep(Duration::from_millis(random_delay)).await;

            println!("URL {}\nDelay {} ms", &url, random_delay);

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

            _permit.forget();
            println!("Task completed ... Going to next one.");
        });

        match task.await {
            Ok(_) => {
                // The task completed successfully
            }
            Err(e) => {
                // The task returned an error
                eprintln!("Task returned an error: {:?}", e);
            }
        }
    }

    Ok(())
}