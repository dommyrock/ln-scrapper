use headless_chrome::Browser;
use rand::Rng;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use tokio::sync::Semaphore;
use tokio::task::{JoinError, JoinSet};
use tokio::time::{sleep, Duration};


#[tokio::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let semaphore = Arc::new(Semaphore::new(4)); // max 4 tasks
    let contents = std::fs::read_to_string("jobs_Success_909_jobs.csv")?;
    let urls: Vec<&str> = contents.split(",").collect();
    let browser = Arc::new(Mutex::new(Browser::default().unwrap()));

    let mut handles:Vec<tokio::task::JoinHandle<()>> = vec![];
    
    urls.into_iter().for_each(|url| {
        let sem_clone = Arc::clone(&semaphore);
        let browser = Arc::clone(&browser);
        let url = url.to_owned();
        let random_delay: u64 = rand::thread_rng().gen_range(25..=80) + 200;

        handles.push(tokio::spawn(async move {
            let permit = sem_clone.clone().acquire_owned().await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(random_delay)).await;

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

            //A JoinHandle detaches the associated task when it is dropped, which means that there is no longer any handle to the task, and no way to join on it.
            drop(permit);
        }));
    });

    for handle in handles {
        println!("Awaiting for all tasks to complete ...");
        handle.await.unwrap();
    }

    Ok(())
}